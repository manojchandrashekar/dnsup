use crate::{lookup, AppConfig, UserConfig};
use core::panic;
use std::{fs::File, io::Write, path::PathBuf, process::exit};

pub fn create_config_and_quit(config_path: &PathBuf, app_config: &AppConfig) {
    static CONFIG_SAMPLE: &str = "
version = \"1\" # For handling changes in config structure

[lookup]
method = \"dig\" # Provision for additional methods in the future
provider = \"opendns\"

[cloudflare]
auth_token = \"cloudflare-auth-token\"
account_id = \"cloudflare-account-id\"
domains = \"comma-separated-domain-names-to-set-A-record-for\"

";
    vlog("Creating config file...", app_config);
    let mut config_file = match File::create(&config_path) {
        Err(why) => panic!("Unable to create config {}: {}", config_path.display(), why),
        Ok(file) => file,
    };

    vlog("Populating config file with sample data...", app_config);
    match config_file.write_all(CONFIG_SAMPLE.as_bytes()) {
        Err(why) => panic!("Unable to create config {}: {}", config_path.display(), why),
        Ok(_) => {
            println!("No config file was found. A sample file was created at: {}. Update the configuration and run dnsup again.", config_path.display());
            vlog("Sample data written successfully.", app_config);
            exit(0);
        }
    }
}

pub fn validate_config(user_config: &mut UserConfig, app_config: &AppConfig) {
    // TODO: Handle config file versioning
    vlog("Validating config...", app_config);
    vlog("Validating config: lookup", app_config);
    match user_config.lookup.method.as_str() {
        "dig" => {
            vlog("Validating config: lookup method", app_config);
            lookup::dig::validate();
            vlog("Validating config: lookup method -- Done", app_config);
            vlog("Validating config: lookup provider", app_config);
            match user_config.lookup.provider.as_str() {
                "opendns" => {
                    lookup::opendns::validate(user_config, app_config);
                    vlog("Validating config: lookup provider -- Done", &app_config);
                }
                _ => {
                    panic!("Unsupported lookup provider! Check config.");
                }
            }
            vlog("Validating config: lookup -- Done", app_config);
        }
        _ => {
            panic!("Unsupported lookup method! Check config.");
        }
    }
}

pub fn vlog(msg: &str, app_config: &AppConfig) {
    if app_config.verbose {
        println!("++LOG: {}", msg);
    }
}

pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
