use std::time::Duration;

use eyre::Error;
use serde::Deserialize;
use torrent_common::Torrent;

#[derive(Clone)]
pub struct Client {
    http: ureq::Agent,
    base_url: String,
    apikey: String,
}

impl Client {
    pub fn new(agent: ureq::Agent, base_url: impl ToString, apikey: impl ToString) -> Self {
        Self {
            http: agent,
            base_url: base_url.to_string() + "/api/v2.0",
            apikey: apikey.to_string(),
        }
    }

    pub fn search(
        &self,
        query: &str,
        categories: Option<&[impl AsRef<str>]>,
        trackers: Option<&[impl AsRef<str>]>,
    ) -> Result<Vec<Result<Torrent, Error>>, Error> {
        let url = format!("{}/indexers/all/results", self.base_url);
        let cookie_exp = self
            .http
            .cookie_store()
            .get_any("jackett.noto.box.ca", "/", "Jackett")
            .unwrap()
            .expires()
            .unwrap()
            .datetime()
            .unwrap()
            .unix_timestamp()
            .to_string();
        let mut request = self
            .http
            .get(&url)
            .query("apikey", &self.apikey)
            .query("Query", query)
            .query("_", &cookie_exp);
        if let Some(categories) = categories {
            request = request.query_pairs(categories.iter().map(|c| ("Category[]", c.as_ref())));
        }
        if let Some(trackers) = trackers {
            request = request.query_pairs(trackers.iter().map(|t| ("Tracker[]", t.as_ref())));
        }
        let response: QueryResult = request.call()?.into_json()?;
        Ok(response
            .results
            .into_iter()
            .map(Torrent::try_from)
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct QueryResult {
    #[serde(alias = "Results")]
    results: Vec<QueryResultTorrent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct QueryResultTorrent {
    title: String,
    size: u64,
    category: Vec<u32>,
    link: String,
    seeders: Option<u16>,
    peers: Option<u16>,
    minimum_ratio: Option<f32>,
    minimum_seed_time: Option<u64>,
}

impl TryFrom<QueryResultTorrent> for torrent_common::Torrent {
    type Error = eyre::Error;
    fn try_from(torrent: QueryResultTorrent) -> Result<Self, Self::Error> {
        Ok(torrent_common::Torrent::new(
            torrent.title,
            torrent.size,
            torrent.category,
            torrent.link,
            torrent.seeders,
            torrent.peers,
            torrent.minimum_ratio,
            torrent.minimum_seed_time.map(Duration::from_secs),
        )?)
    }
}
