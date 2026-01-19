use std::env;

pub struct Config {
    pub xray_path: String,
    pub config_file_name: String,
    pub config_path: String,
    pub socks_start_port: u16,
    pub server_port: u16,
    pub default_user_level: u32,
    pub default_fingerprint: String,
    pub loopback_address: String,
    pub subscription_user_agent: String,
    pub filter_keywords: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let xray_path = env::var("XRAY_PATH").unwrap_or_else(|_| "/opt/homebrew/bin/xray".to_string());
        let config_file_name = env::var("CONFIG_FILE_NAME").unwrap_or_else(|_| "xray.json".to_string());
        
        let current_dir = env::current_dir().unwrap_or_else(|_| ".".into());
        let default_config_path = current_dir.join(&config_file_name).to_string_lossy().to_string();
        
        let config_path = env::var("CONFIG_PATH").unwrap_or(default_config_path);
        
        let socks_start_port = env::var("SOCKS_START_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50000);
            
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3123);
            
        let default_user_level = env::var("DEFAULT_USER_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8);
            
        let default_fingerprint = env::var("DEFAULT_FINGERPRINT").unwrap_or_else(|_| "safari".to_string());
        let loopback_address = env::var("LOOPBACK_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
        let subscription_user_agent = env::var("SUBSCRIPTION_USER_AGENT").unwrap_or_else(|_| "v2rayNG/1.8.5".to_string());
        
        let filter_keywords_str = env::var("FILTER_KEYWORDS").unwrap_or_default();
        let filter_keywords = filter_keywords_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Config {
            xray_path,
            config_file_name,
            config_path,
            socks_start_port,
            server_port,
            default_user_level,
            default_fingerprint,
            loopback_address,
            subscription_user_agent,
            filter_keywords,
        }
    }
}
