use anyhow::{anyhow, Result};
use url::Url;

#[derive(Debug, Clone)]
pub struct VlessConfig {
    pub uuid: String,
    pub address: String,
    pub port: u16,
    pub type_: String,
    pub encryption: String,
    pub security: String,
    pub flow: String,
    pub sni: String,
    pub pbk: String,
    pub sid: String,
    pub fp: String,
    pub name: String,
    pub path: String,
    pub host: String,
}

pub fn parse_vless_link(link: &str) -> Result<VlessConfig> {
    if !link.starts_with("vless://") {
        return Err(anyhow!("Not a vless link"));
    }

    let url = Url::parse(link).map_err(|e| anyhow!("Failed to parse URL: {}", e))?;
    let uuid = url.username().to_string();
    
    let host_str = url.host_str().ok_or_else(|| anyhow!("Missing host"))?;
    let port = url.port().ok_or_else(|| anyhow!("Missing port"))?;
    let address = host_str.to_string();

    let query_map: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    let type_ = query_map.get("type").cloned().unwrap_or_else(|| "tcp".to_string());
    let security = query_map.get("security").cloned().unwrap_or_default();
    let mut flow = query_map.get("flow").cloned().unwrap_or_default();

    // Auto-fill flow for Reality/TLS + TCP if missing
    if security == "reality" && flow.is_empty() {
        flow = "xtls-rprx-vision".to_string();
    }

    let encryption = query_map.get("encryption").cloned().unwrap_or_else(|| "none".to_string());
    let sni = query_map.get("sni").cloned().unwrap_or_default();
    let pbk = query_map.get("pbk").cloned().unwrap_or_default();
    let sid = query_map.get("sid").cloned().unwrap_or_default();
    let fp = query_map.get("fp").cloned().unwrap_or_default();
    let path = query_map.get("path").cloned().unwrap_or_default();
    let host = query_map.get("host").cloned().unwrap_or_default();

    let name = url.fragment()
        .map(|f| urlencoding::decode(f).map(|s| s.into_owned()).unwrap_or_else(|_| f.to_string()))
        .unwrap_or_else(|| "proxy".to_string());

    Ok(VlessConfig {
        uuid,
        address,
        port,
        type_,
        encryption,
        security,
        flow,
        sni,
        pbk,
        sid,
        fp,
        name,
        path,
        host,
    })
}
