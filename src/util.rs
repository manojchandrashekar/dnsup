use crate::{lookup, UserConfig};
use anyhow::{bail, Result};
use std::{fs::File, io::Write, path::PathBuf, process::exit};

pub fn create_config_and_quit(config_path: PathBuf) -> ! {
    static CONFIG_SAMPLE: &str = include_str!("../dnsup.sample.toml");

    log::info!("Creating config file...");
    let mut config_file = File::create(&config_path)
        .map_err(|e| {
            log::error!("Unable to create config {}: {}", config_path.display(), e);
            eprintln!("Unable to create config {}: {}", config_path.display(), e);
            std::process::exit(1);
        })
        .unwrap();

    log::info!("Populating config file with sample data...");
    config_file
        .write_all(CONFIG_SAMPLE.as_bytes())
        .map_err(|e| {
            log::error!("Unable to write config {}: {}", config_path.display(), e);
            eprintln!("Unable to write config {}: {}", config_path.display(), e);
            std::process::exit(1);
        })
        .unwrap();

    log::info!("Sample data written successfully.");
    println!("No config file was found. A sample file was created at: {}. Update the configuration and run dnsup again.", config_path.display());

    exit(0);
}

pub fn validate_config(user_config: &mut UserConfig) -> Result<()> {
    // TODO: Handle config file versioning
    log::info!("Validating config...");
    log::info!("Validating config: lookup");
    match user_config.lookup.method.as_str() {
        "dig" => {
            log::info!("Validating config: lookup method");
            lookup::dig::validate()?;
            log::info!("Validating config: lookup method -- Done");
            log::info!("Validating config: lookup provider");
            match user_config.lookup.provider.as_str() {
                "opendns" => {
                    lookup::opendns::validate(user_config)?;
                    log::info!("Validating config: lookup provider -- Done");
                }
                _ => {
                    bail!("Unsupported lookup provider! Check config.");
                }
            }
            log::info!("Validating config: lookup -- Done");
        }
        _ => {
            bail!("Unsupported lookup method! Check config.");
        }
    }
    Ok(())
}

pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
