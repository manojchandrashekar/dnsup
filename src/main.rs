use argparse::{ArgumentParser, Store, StoreTrue};
use std::fs;
use std::net::Ipv4Addr;
// use dotenv::dotenv;
// use std::env;
use std::path::PathBuf;
use std::process::exit;
mod api;
mod lookup;
mod util;
use serde::Deserialize;

use crate::util::vlog;

pub struct AppConfig {
    verbose: bool,
    config_file: PathBuf,
    custom_config: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            verbose: Default::default(),
            config_file: dirs::config_dir()
                .map(|path| path.join(".dnsup.toml"))
                .unwrap_or_else(|| "/tmp/dnsup.toml".into()),
            custom_config: dirs::config_dir()
                .map(|path| path.join(".dnsup.toml"))
                .unwrap_or_else(|| "/tmp/dnsup.toml".into()),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Lookup {
    method: String,
    provider: String,
}

#[derive(Deserialize, Debug)]
pub struct Cloudflare {
    auth_token: String,
    account_id: String,
    domains: String,
}

#[derive(Deserialize, Debug)]
pub struct UserConfig {
    version: String,
    lookup: Lookup,
    cloudflare: Option<Cloudflare>,
    ip: std::net::Ipv4Addr,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            version: "0".to_string(),
            lookup: Lookup {
                method: "dig".to_string(),
                provider: "opendns".to_string(),
            },
            cloudflare: Default::default(),
            ip: Ipv4Addr::new(0, 0, 0, 0),
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Default app config
    let mut app_config = AppConfig::default();

    // Default user config
    let mut user_config = UserConfig::default();

    // TODO: Add support to set config in ENV
    // dotenv().ok();
    // let envs = env::vars();

    // Parse command line arguments
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Keep your DNS up!");
        ap.refer(&mut app_config.verbose).add_option(
            &["-v", "--verbose"],
            StoreTrue,
            "Verbose output",
        );
        ap.refer(&mut app_config.config_file).add_option(
            &["-c", "--config"],
            Store,
            "Specify config file to use",
        );
        ap.parse_args_or_exit();
    }

    vlog("Reading config file...", &app_config);
    let config_file = if app_config.custom_config != app_config.config_file {
        &app_config.custom_config
    } else {
        &app_config.config_file
    };
    if app_config.config_file.is_file() {
        let contents: String = fs::read_to_string(config_file).expect("Error reading config file");

        vlog(
            "Config file read successful. Parsing contents...",
            &app_config,
        );

        user_config = toml::from_str(contents.as_str())?;

        vlog(
            format!(
                "Config parsed successfully. (Version: {})",
                user_config.version
            )
            .as_str(),
            &app_config,
        );
        util::validate_config(&mut user_config, &app_config);
    } else if app_config.custom_config != app_config.config_file {
        vlog("Config file not found, creating one...", &app_config);
        util::create_config_and_quit(&app_config.config_file, &app_config);
    }

    if user_config.cloudflare.is_some() {
        vlog("Validating config: cloudflare", &app_config);
        match api::cloudflare::validate(&mut user_config, &app_config).await {
            Ok(_t) => {
                vlog("Validating config: cloudflare -- Done", &app_config);
                vlog("Processing cloudflare...", &app_config);
                match api::cloudflare::execute(&user_config, &app_config).await {
                    Ok(_t) => {}
                    Err(_e) => {}
                };
            }
            Err(_e) => {}
        };
    } else {
        println!("No domain configurations found! Check config.");
        exit(0);
    }

    Ok(())
}
