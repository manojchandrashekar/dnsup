use std::{net::Ipv4Addr, process::Command};

pub fn validate() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("dig -v")
        .output()
        .expect("Failed to validate lookup method: dig. Verify that `dig` is installed and available in PATH.");
    if !output.status.success() {
        panic!(
            "{}",
            format!(
                "Failed to validate lookup method: dig. \nError: {}",
                String::from_utf8(output.stderr).unwrap()
            )
        )
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
