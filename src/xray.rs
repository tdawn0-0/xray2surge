use crate::config::Config;
use crate::vless::VlessConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct XrayConfig {
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub routing: Routing,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Inbound {
    pub tag: String,
    pub port: u16,
    pub listen: String,
    pub protocol: String,
    pub settings: InboundSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InboundSettings {
    pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Outbound {
    pub tag: String,
    pub protocol: String,
    pub settings: OutboundSettings,
    #[serde(rename = "streamSettings")]
    pub stream_settings: StreamSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutboundSettings {
    pub vnext: Vec<VNext>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VNext {
    pub address: String,
    pub port: u16,
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub flow: String,
    pub encryption: String,
    pub level: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamSettings {
    pub network: String,
    pub security: String,
    #[serde(rename = "realitySettings", skip_serializing_if = "Option::is_none")]
    pub reality_settings: Option<RealitySettings>,
    #[serde(rename = "tlsSettings", skip_serializing_if = "Option::is_none")]
    pub tls_settings: Option<TlsSettings>,
    #[serde(rename = "wsSettings", skip_serializing_if = "Option::is_none")]
    pub ws_settings: Option<WsSettings>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RealitySettings {
    pub show: bool,
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "shortId")]
    pub short_id: String,
    pub fingerprint: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TlsSettings {
    #[serde(rename = "allowInsecure")]
    pub allow_insecure: bool,
    pub fingerprint: String,
    #[serde(rename = "serverName")]
    pub server_name: String,
    pub show: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WsSettings {
    pub headers: WsHeaders,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WsHeaders {
    #[serde(rename = "Host")]
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Routing {
    pub rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "inboundTag")]
    pub inbound_tag: Vec<String>,
    #[serde(rename = "outboundTag")]
    pub outbound_tag: String,
}

pub fn generate_xray_config(configs: &[VlessConfig], app_config: &Config) -> XrayConfig {
    let inbounds = configs
        .iter()
        .enumerate()
        .map(|(index, _)| Inbound {
            tag: format!("proxy-{}-in", index),
            port: app_config.socks_start_port + (index as u16) + 1,
            listen: app_config.loopback_address.clone(),
            protocol: "socks".to_string(),
            settings: InboundSettings {
                auth: "none".to_string(),
            },
        })
        .collect();

    let outbounds = configs
        .iter()
        .enumerate()
        .map(|(index, cfg)| {
            let mut stream_settings = StreamSettings {
                network: cfg.type_.clone(),
                security: cfg.security.clone(),
                reality_settings: None,
                tls_settings: None,
                ws_settings: None,
            };

            if cfg.security == "reality" {
                stream_settings.reality_settings = Some(RealitySettings {
                    show: false,
                    server_name: cfg.sni.clone(),
                    public_key: cfg.pbk.clone(),
                    short_id: cfg.sid.clone(),
                    fingerprint: if cfg.fp.is_empty() {
                        app_config.default_fingerprint.clone()
                    } else {
                        cfg.fp.clone()
                    },
                });
            }

            if cfg.security == "tls" {
                stream_settings.tls_settings = Some(TlsSettings {
                    allow_insecure: false,
                    fingerprint: if cfg.fp.is_empty() {
                        app_config.default_fingerprint.clone()
                    } else {
                        cfg.fp.clone()
                    },
                    server_name: cfg.sni.clone(),
                    show: false,
                });
            }

            if cfg.type_ == "ws" {
                stream_settings.ws_settings = Some(WsSettings {
                    headers: WsHeaders {
                        host: cfg.host.clone(),
                    },
                    path: cfg.path.clone(),
                });
            }

            Outbound {
                tag: format!("proxy-{}-out", index),
                protocol: "vless".to_string(),
                settings: OutboundSettings {
                    vnext: vec![VNext {
                        address: cfg.address.clone(),
                        port: cfg.port,
                        users: vec![User {
                            id: cfg.uuid.clone(),
                            flow: cfg.flow.clone(),
                            encryption: cfg.encryption.clone(),
                            level: app_config.default_user_level,
                        }],
                    }],
                },
                stream_settings,
            }
        })
        .collect();

    let rules = configs
        .iter()
        .enumerate()
        .map(|(index, _)| Rule {
            type_: "field".to_string(),
            inbound_tag: vec![format!("proxy-{}-in", index)],
            outbound_tag: format!("proxy-{}-out", index),
        })
        .collect();

    XrayConfig {
        inbounds,
        outbounds,
        routing: Routing { rules },
    }
}
