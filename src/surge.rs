use crate::config::Config;
use crate::hysteria2::Hysteria2Config;
use crate::vless::VlessConfig;
use uuid::Uuid;

pub fn generate_surge_list(
    vless_configs: &[VlessConfig],
    hysteria2_configs: &[Hysteria2Config],
    app_config: &Config,
) -> String {
    if vless_configs.is_empty() && hysteria2_configs.is_empty() {
        return String::new();
    }

    let mut output = String::from("[Proxy]\n");

    // Add a fake proxy with a random name to force Surge to restart Xray on subscription update
    let random_name = format!("xray_{}", Uuid::new_v4().simple());
    output.push_str(&format!(
        "{} = external, exec = \"{}\", local-port = {}, args = \"run\", args = \"-c\", args = \"{}\"\n",
        random_name,
        app_config.xray_path,
        app_config.socks_start_port,
        app_config.config_path
    ));

    // Output VLESS proxies (via Xray socks5)
    for (index, cfg) in vless_configs.iter().enumerate() {
        let port = app_config.socks_start_port + (index as u16) + 1;
        // Sanitize name for Surge
        let name = cfg
            .name
            .trim()
            .replace(|c: char| c.is_whitespace() || c == ',', "_");

        if index == 0 {
            // First one is the external one that starts Xray
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
                name, app_config.loopback_address, port
            ));
        }
    }

    // Output Hysteria2 proxies (native Surge support)
    for cfg in hysteria2_configs {
        let name = cfg
            .name
            .trim()
            .replace(|c: char| c.is_whitespace() || c == ',', "_");

        let mut line = format!(
            "{} = hysteria2, {}, {}, password={}",
            name, cfg.address, cfg.port, cfg.password
        );

        if cfg.insecure {
            line.push_str(", skip-cert-verify=true");
        }

        if !cfg.sni.is_empty() {
            line.push_str(&format!(", sni={}", cfg.sni));
        }

        line.push('\n');
        output.push_str(&line);
    }

    output
}
