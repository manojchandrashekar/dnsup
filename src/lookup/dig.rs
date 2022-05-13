use std::{net::Ipv4Addr, process::Command};

pub fn validate() {
    match Command::new("dig").arg("-v").output() {
        Err(e) => {
            log::error!(
                "{}. Verify that `dig` is installed and available in PATH.",
                e
            );
            std::process::exit(1);
        }
        Ok(output) => {
            if !output.status.success() {
                log::error!("Failed to validate lookup method: dig");
                log::error!("{}", String::from_utf8(output.stderr).unwrap());
                std::process::exit(1);
            }
        }
    }
}

pub fn execute(options: String, domain: String, resolver: String) -> Ipv4Addr {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("dig {} {} @{}", options, domain, resolver))
        .output()
        .expect("Failed to fetch current IP from `opendns`. Check network.");
    if !output.status.success() {
        panic!(
            "{}",
            format!(
                "Failed to fetch current IP. Check network. \nError: {}",
                String::from_utf8(output.stderr).unwrap()
            )
        )
    }
    return String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
}
