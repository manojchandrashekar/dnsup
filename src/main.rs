use anyhow::{anyhow, Result};
use api::cloudflare::Cloudflare;
use argparse::{ArgumentParser, Store, StoreTrue};
use std::fs;
use std::net::Ipv4Addr;
// use dotenv::dotenv;
// use std::env;
use std::process::exit;
mod api;
mod lookup;
mod util;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Lookup {
    method: String,
    provider: String,
}

#[derive(Deserialize, Debug)]
pub struct UserConfig {
    version: String,
    lookup: Lookup,
    cloudflare: Option<Cloudflare>,
    ip: Ipv4Addr,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut verbose = false;
    let mut config_file = dirs::config_dir()
        .map(|path| path.join("dnsup.toml"))
        .unwrap_or_else(|| "/tmp/dnsup.toml".into());

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Keep your DNS up!");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Verbose output");
        ap.refer(&mut config_file).add_option(
            &["-c", "--config"],
            Store,
            "Specify config file to use",
        );
        ap.parse_args_or_exit();
    }

    if verbose {
        simple_logger::init_with_level(log::Level::Info).expect("Logger failed to initialize!");
    }

    if !config_file.is_file() {
        log::info!(
            "Config file not found, creating one at {}",
            config_file.display()
        );
        util::create_config_and_quit(config_file);
    }

    // TODO: Add support to set config in ENV
    // dotenv().ok();
    // let envs = env::vars();

    // Parse command line arguments

    log::info!("Reading config file - {}", config_file.display());

    let contents = fs::read_to_string(&config_file)
        .map_err(|e| anyhow!("Reading config file {} - {e}", config_file.display()))?;

    log::info!("Config file read successful. Parsing contents...");

    let mut config: UserConfig = toml::from_str(contents.as_str())
        .map_err(|e| anyhow!("Parsing config file {} - {e}", config_file.display()))?;
    util::validate_config(&mut config)?;
    log::info!("Config parsed successfully. (Version: {})", config.version);

    if let Some(mut cloudflare) = config.cloudflare {
        log::info!("Validating config: cloudflare");
        cloudflare.validate().await?;
        log::info!("Processing cloudflare...");
        cloudflare.execute(config.ip).await?;
    } else {
        println!("No domain configurations found! Check config.");
        exit(0);
    }

    Ok(())
}
