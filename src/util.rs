use crate::{lookup, UserConfig};
use core::panic;
use std::{fs::File, io::Write, path::PathBuf, process::exit};

pub fn create_config_and_quit(config_path: PathBuf) -> ! {
    static CONFIG_SAMPLE: &str = r#"
version = "1" # For handling changes in config structure
ip = "0.0.0.0"

[lookup]
method = "dig" # Provision for additional methods in the future
provider = "opendns"

[cloudflare]
auth_token = "cloudflare-auth-token"
account_id = "cloudflare-account-id"
domains = "comma-separated-domain-names-to-set-A-record-for"
"#;
    log::info!("Creating config file...");
    let mut config_file = match File::create(&config_path) {
        Err(why) => panic!("Unable to create config {}: {}", config_path.display(), why),
        Ok(file) => file,
    };

    log::info!("Populating config file with sample data...");
    match config_file.write_all(CONFIG_SAMPLE.as_bytes()) {
        Err(why) => panic!("Unable to create config {}: {}", config_path.display(), why),
        Ok(_) => {
            log::info!("Sample data written successfully.");
            println!("No config file was found. A sample file was created at: {}. Update the configuration and run dnsup again.", config_path.display());
            exit(0);
        }
    }
}

pub fn validate_config(user_config: &mut UserConfig) {
    // TODO: Handle config file versioning
    log::info!("Validating config...");
    log::info!("Validating config: lookup");
    match user_config.lookup.method.as_str() {
        "dig" => {
            log::info!("Validating config: lookup method");
            lookup::dig::validate();
            log::info!("Validating config: lookup method -- Done");
            log::info!("Validating config: lookup provider");
            match user_config.lookup.provider.as_str() {
                "opendns" => {
                    lookup::opendns::validate(user_config);
                    log::info!("Validating config: lookup provider -- Done");
                }
                _ => {
                    panic!("Unsupported lookup provider! Check config.");
                }
            }
            log::info!("Validating config: lookup -- Done");
        }
        _ => {
            panic!("Unsupported lookup method! Check config.");
        }
    }
}

pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
