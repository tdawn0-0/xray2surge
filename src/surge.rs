use crate::config::Config;
use crate::vless::VlessConfig;

pub fn generate_surge_list(configs: &[VlessConfig], app_config: &Config) -> String {
    if configs.is_empty() {
        return String::new();
    }

    let mut output = String::from("[Proxy]\n");

    for (index, cfg) in configs.iter().enumerate() {
        let port = app_config.socks_start_port + (index as u16) + 1;
        // Sanitize name for Surge
        let name = cfg.name.trim().replace(|c: char| c.is_whitespace() || c == ',', "_");

        if index == 0 {
            // First one is the external one that starts Xray
            // Note: We need to escape quotes for the args if needed, but here simple replacement is likely enough.
            output.push_str(&format!(
                "{} = external, exec = \"{}\", local-port = {}, args = \"run\", args = \"-c\", args = \"{}\"\n",
                name,
                app_config.xray_path,
                port,
                app_config.config_path
            ));
        } else {
            output.push_str(&format!(
                "{} = socks5, {}, {}\n",
                name,
                app_config.loopback_address,
                port
            ));
        }
    }

    output
}
