use anyhow::{anyhow, bail, Result};
use std::{net::Ipv4Addr, process::Command};

pub fn validate() -> Result<()> {
    let output = Command::new("dig").arg("-v").output().map_err(|e| {
        log::error!("Failed to execute command: dig -v - {e}");
        anyhow!("Failed to execute command: dig -v - {e}")
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        log::error!("Failed to validate lookup method dig - {}", stderr);
        bail!("Failed to validate lookup method dig - {}", stderr);
    }

    log::info!("Lookup method dig validated successfully");
    Ok(())
}

pub fn execute(options: String, domain: String, resolver: String) -> Result<Ipv4Addr> {
    let output = Command::new("dig")
        .arg(options)
        .arg(domain)
        .arg(format!("@{}", resolver))
        .output()
        .map_err(|e| anyhow!("Failed to fetch current IP from `opendns`: {e}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout)?
            .trim()
            .parse()
            .map_err(|e| anyhow!("Error parsing IPv4 from dig - {e}"))
    } else {
        bail!(
            "Failed to fetch current IP. Check network - {}",
            String::from_utf8(output.stderr)?
        )
    }
}
