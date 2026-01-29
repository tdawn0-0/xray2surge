use anyhow::{anyhow, Result};
use url::Url;

#[derive(Debug, Clone)]
pub struct Hysteria2Config {
    pub password: String,
    pub address: String,
    pub port: u16,
    pub insecure: bool,
    pub sni: String,
    pub name: String,
}

pub fn parse_hysteria2_link(link: &str) -> Result<Hysteria2Config> {
    if !link.starts_with("hysteria2://") && !link.starts_with("hy2://") {
        return Err(anyhow!("Not a hysteria2 link"));
    }

    let url = Url::parse(link).map_err(|e| anyhow!("Failed to parse URL: {}", e))?;

    // Password is in the username position
    let password = url.username().to_string();

    let host_str = url.host_str().ok_or_else(|| anyhow!("Missing host"))?;
    let port = url.port().unwrap_or(443);
    let address = host_str.to_string();

    let query_map: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    let insecure = query_map
        .get("insecure")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    let sni = query_map.get("sni").cloned().unwrap_or_default();

    let name = url
        .fragment()
        .map(|f| {
            urlencoding::decode(f)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| f.to_string())
        })
        .unwrap_or_else(|| "hysteria2-proxy".to_string());

    Ok(Hysteria2Config {
        password,
        address,
        port,
        insecure,
        sni,
        name,
    })
}
