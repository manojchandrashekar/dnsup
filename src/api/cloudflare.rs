use crate::util::remove_whitespace;
use anyhow::{bail, Result};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::Ipv4Addr};

static CF_API_ENDPOINT: &str = "https://api.cloudflare.com/client/v4/";

#[derive(Deserialize, Debug)]
pub struct Cloudflare {
    auth_token: String,
    account_id: String,
    domains: String,
}

#[derive(Deserialize, Debug)]
struct ZoneResponse {
    result: Vec<ZoneResponseResult>,
}

#[derive(Deserialize, Debug)]
struct DNSResponse {
    result: Vec<ZoneResponseResult>,
}

#[derive(Deserialize, Debug)]
struct ZoneResponseResult {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct DNSRequestBody {
    #[serde(rename(serialize = "type"))]
    record_type: String,
    name: String,
    content: String,
}

#[derive(Debug)]
struct DNSUpdates {
    domain: String,
    zone: Option<ZoneResponseResult>,
    id: Option<String>,
}

impl Cloudflare {
    pub async fn validate(&mut self) -> Result<()> {
        log::info!("Validating config: cloudflare auth token");
        let verify_ep = format!("{}user/tokens/verify", CF_API_ENDPOINT);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(format!("Bearer {}", self.auth_token).as_str()).unwrap(),
        );
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::new();
        let res = client.get(verify_ep).headers(headers).send().await?;
        if res.status() != StatusCode::OK {
            log::error!("{}", res.text().await.unwrap());
            bail!("Cloudflare token invalid! Check config.")
        } else {
            log::info!("Validating config: cloudflare auth token -- Done");
            Ok(())
        }
    }

    pub async fn execute(&self, ip: Ipv4Addr) -> Result<()> {
        log::info!("Processing cloudflare: get all zones");
        let zones_ep = format!(
            "{}zones?status=active&account.id={}&page=1&per_page=100&order=status&match=all",
            CF_API_ENDPOINT, self.account_id
        );
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(format!("Bearer {}", self.auth_token).as_str()).unwrap(),
        );
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::new();
        log::info!("API: {}\n", zones_ep.as_str());
        let res = client.get(zones_ep).headers(headers).send().await?;
        if res.status() != StatusCode::OK {
            log::info!("{}", res.text().await.unwrap());
            bail!("Unable to query Zone list from cloudflare. Check if permissions are set correctly for the auth token.");
        }
        let res_text = res.text().await.unwrap();
        let zone_response: ZoneResponse = serde_json::from_str(res_text.as_str())?;
        let domains = remove_whitespace(&self.domains.as_str().to_lowercase());
        let domains_vec = domains.split(',').collect::<Vec<&str>>();
        let mut dns_vec: Vec<DNSUpdates> = Vec::new();
        for domain in &domains_vec {
            let mut names: Vec<String> = Vec::new();
            let domain_vec: Vec<&str> = domain.split('.').rev().collect();
            let mut d_iter = domain_vec.iter();
            let mut dn = d_iter.next();
            let mut d = dn.unwrap().to_string();
            loop {
                dn = d_iter.next();
                if dn.is_none() {
                    break;
                }
                let d1 = dn.unwrap();
                d = format!("{}.{}", d1, d);
                names.push(d.to_string());
            }
            let zone = zone_response
                .result
                .iter()
                .find(|z| names.contains(&z.name));
            if zone.is_some() {
                let zone_str = ZoneResponseResult {
                    name: (*zone.unwrap().name).to_string(),
                    id: (*zone.unwrap().id).to_string(),
                };
                let dns: DNSUpdates = DNSUpdates {
                    domain: domain.to_string(),
                    zone: Some(zone_str),
                    id: None,
                };
                dns_vec.push(dns);
            } else {
                log::error!("Unable to find the zone for: {}", domain.to_string())
            }
        }

        // TODO: Parallelize the API calls.
        for mut dns in dns_vec.iter_mut() {
            if dns.zone.is_none() {
                log::error!("[{}]: Couldn't find Zone ID. Make sure the domain exists in your cloudflare account.", dns.domain);
                continue;
            }
            let zones_ep = format!(
                "{}zones/{}/dns_records?type=A&name={}&page=1&per_page=100&order=type&direction=desc&match=all",
                CF_API_ENDPOINT,
                dns.zone.as_ref().unwrap().id,
                &dns.domain
                );
            let mut headers = header::HeaderMap::new();
            headers.insert(
                "Authorization",
                header::HeaderValue::from_str(format!("Bearer {}", self.auth_token).as_str())
                    .unwrap(),
            );
            headers.insert(
                "Content-Type",
                header::HeaderValue::from_static("application/json"),
            );
            let client = reqwest::Client::new();
            log::info!("API: {}\n", zones_ep.as_str());
            let res = client.get(zones_ep).headers(headers).send().await?;
            let res_status = res.status().as_u16();
            let res_text = res.text().await.unwrap();

            let dns_res: DNSResponse = serde_json::from_str(res_text.as_str())?;
            if res_status != StatusCode::OK {
                log::error!(
                    "[{}]: Unable to query DNS list from cloudflare. Try again.",
                    dns.domain
                );
            } else {
                dns.id = Some((dns_res.result[0].id).to_string());
            };
        }
        for dns in dns_vec.into_iter() {
            if dns.id.is_none() {
                log::error!("[{}]: Couldn't find DNS ID for `A` record. Make sure an `A` record exists for the domain in cloudflare.", dns.domain);
                continue;
            }
            let dns_patch_ep = format!(
                "{}zones/{}/dns_records/{}",
                CF_API_ENDPOINT,
                dns.zone.as_ref().unwrap().id,
                dns.id.unwrap()
            );
            let mut headers = header::HeaderMap::new();
            headers.insert(
                "Authorization",
                header::HeaderValue::from_str(format!("Bearer {}", self.auth_token).as_str())
                    .unwrap(),
            );
            headers.insert(
                "Content-Type",
                header::HeaderValue::from_static("application/json"),
            );
            let client = reqwest::Client::new();
            let mut body = HashMap::new();
            body.insert("type", "A".to_string());
            body.insert("name", dns.domain.clone());
            body.insert("content", ip.to_string());
            log::info!("Updating `A` record for {}", &dns.domain);
            log::info!("API: {}\n{:?}", dns_patch_ep.as_str(), &body);
            let res = client
                .patch(dns_patch_ep)
                .json(&body)
                .headers(headers)
                .send()
                .await?;
            let res_status = res.status().as_u16();
            let res_text = res.text().await.unwrap();

            if res_status != StatusCode::OK {
                log::info!("{}", res_text);
                log::error!("[{}]: Unable to update the DNS `A` record.", dns.domain);
            };
        }
        Ok(())
    }
}
