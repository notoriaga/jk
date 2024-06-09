mod client;

use eyre::Context;
use gumdrop::Options;

use client::Client;

#[derive(Options)]
struct Cli {
    #[options(free)]
    query: String,

    #[options(help = "print help message")]
    help: bool,

    #[options(help = "list of trackers to search")]
    trackers: Vec<String>,

    #[options(help = "list of categories to search")]
    categories: Vec<String>,

    #[options(no_short, help = "jacket base url, falls back to JACKET_URL env var")]
    jacket_url: Option<String>,

    #[options(no_short, help = "jacket api key, falls back to JACKET_API env var")]
    jacket_apikey: Option<String>,

    #[options(
        no_short,
        help = "jacket password, falls back to JACKET_PASSWORD env var"
    )]
    jacket_password: Option<String>,
}

fn main() -> Result<(), eyre::Error> {
    let cli = Cli::parse_args_default_or_exit();

    let url = if let Some(url) = cli.jacket_url {
        url
    } else {
        std::env::var("JACKET_URL").wrap_err("getting JACKET_URL")?
    };

    let apikey = if let Some(apikey) = cli.jacket_apikey {
        apikey
    } else {
        std::env::var("JACKET_APIKEY").wrap_err("getting JACKET_APIKEY")?
    };

    let password = if let Some(password) = cli.jacket_password {
        password
    } else {
        std::env::var("JACKET_PASSWORD").wrap_err("getting JACKET_PASSWORD")?
    };

    let agent = get_authed_agent(&url, &password)?;

    let client = Client::new(agent, url, apikey);
    let categories = (!cli.categories.is_empty()).then_some(cli.categories.as_slice());
    let trackers = (!cli.trackers.is_empty()).then_some(cli.trackers.as_slice());

    for result in client.search(&cli.query, categories, trackers)? {
        match result {
            Ok(r) => println!("{:?}", r),
            Err(e) => eprintln!("{e}"),
        }
    }

    Ok(())
}

fn get_authed_agent(jacket_url: &str, admin_password: &str) -> Result<ureq::Agent, eyre::Error> {
    let agent = ureq::AgentBuilder::new().https_only(true).build();
    agent
        .post(&format!("{}/UI/Dashboard", jacket_url))
        .send_form(&[("password", admin_password)])?;
    let domain = jacket_url.strip_prefix("https://").unwrap_or(jacket_url);
    if agent.cookie_store().contains(domain, "/", "Jackett") {
        Ok(agent)
    } else {
        Err(eyre::eyre!("failed to get auth cookie"))
    }
}
