use crate::hysteria2::{parse_hysteria2_link, Hysteria2Config};
use crate::surge::generate_surge_list;
use crate::vless::parse_vless_link;
use crate::xray::generate_xray_config;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::USER_AGENT;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;

pub async fn fetch_subscription(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // Support multiple URLs: either comma-separated in a single 'url' param,
    // or multiple 'url' params (axum merges them with comma)
    let urls_param = match params.get("url") {
        Some(url) => url,
        None => return (StatusCode::BAD_REQUEST, "Missing 'url' query parameter").into_response(),
    };

    // Split by comma to support multiple URLs
    let urls: Vec<&str> = urls_param
        .split(',')
        .map(|u| u.trim())
        .filter(|u| !u.is_empty())
        .collect();

    if urls.is_empty() {
        return (StatusCode::BAD_REQUEST, "No valid URLs provided").into_response();
    }

    let include_hysteria = parse_include_hysteria(&params);

    let client = reqwest::Client::new();
    let mut all_vless_configs = Vec::new();
    let mut all_hysteria2_configs: Vec<Hysteria2Config> = Vec::new();

    // Fetch and parse all URLs
    for target_url in &urls {
        println!("Fetching subscription from: {}", target_url);

        let resp = match client
            .get(*target_url)
            .header(USER_AGENT, &state.config.subscription_user_agent)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to fetch subscription from {}: {}", target_url, e);
                continue; // Skip this URL and try the next one
            }
        };

        if !resp.status().is_success() {
            eprintln!(
                "Failed to fetch subscription from {}: {}",
                target_url,
                resp.status()
            );
            continue;
        }

        let encoded_body = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to read response body from {}: {}", target_url, e);
                continue;
            }
        };

        // Decode Base64
        let decoded_body = match decode_base64_content(&encoded_body) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to decode base64 from {}: {}", target_url, e);
                continue;
            }
        };

        let links: Vec<&str> = decoded_body
            .split('\n')
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        for link in links {
            // Try parsing as VLESS first
            if let Ok(config) = parse_vless_link(link) {
                let is_filtered = state
                    .config
                    .filter_keywords
                    .iter()
                    .any(|w| config.name.contains(w));
                if is_filtered {
                    println!("Filtered out proxy: {}", config.name);
                    continue;
                }
                all_vless_configs.push(config);
            }
            // Try parsing as Hysteria2
            else if include_hysteria {
                if let Ok(config) = parse_hysteria2_link(link) {
                    let is_filtered = state
                        .config
                        .filter_keywords
                        .iter()
                        .any(|w| config.name.contains(w));
                    if is_filtered {
                        println!("Filtered out proxy: {}", config.name);
                        continue;
                    }
                    all_hysteria2_configs.push(config);
                }
            }
        }
    }

    if all_vless_configs.is_empty() && all_hysteria2_configs.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            "No valid proxy links found from any URL",
        )
            .into_response();
    }

    // Handle duplicate names by adding hostname suffix
    let vless_configs = deduplicate_proxy_names(all_vless_configs);

    let xray_config = generate_xray_config(&vless_configs, &state.config);

    // Write Xray config to file
    let config_json = match serde_json::to_string_pretty(&xray_config) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize Xray config: {}", e),
            )
                .into_response()
        }
    };

    if let Err(e) = fs::write(&state.config.config_path, config_json).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write Xray config to file: {}", e),
        )
            .into_response();
    }

    println!("Written Xray config to {}", state.config.config_path);

    let hysteria2_configs = if include_hysteria {
        &all_hysteria2_configs[..]
    } else {
        &[]
    };
    let surge_list = generate_surge_list(&vless_configs, hysteria2_configs, &state.config);

    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; charset=utf-8",
        )],
        surge_list,
    )
        .into_response()
}

/// Parse the `hysteria` query parameter. Defaults to `true` (include Hysteria2 proxies).
/// Accepts: true/false, 1/0, yes/no (case-insensitive).
fn parse_include_hysteria(params: &HashMap<String, String>) -> bool {
    match params.get("hysteria").map(|s| s.to_lowercase()) {
        Some(v) if matches!(v.as_str(), "false" | "0" | "no") => false,
        Some(v) if matches!(v.as_str(), "true" | "1" | "yes") => true,
        None => true,
        Some(_) => true,
    }
}

fn decode_base64_content(content: &str) -> anyhow::Result<String> {
    let trimmed = content.trim();
    // Sometimes padding might be missing or whitespace issues
    let decoded_bytes = general_purpose::STANDARD.decode(trimmed)?;
    String::from_utf8(decoded_bytes).map_err(|e| anyhow::anyhow!(e))
}

use crate::vless::VlessConfig;

/// Deduplicate proxy names by adding hostname suffix for configs with the same name.
/// If multiple proxies have the same name but different hostnames, append the hostname to distinguish them.
fn deduplicate_proxy_names(mut configs: Vec<VlessConfig>) -> Vec<VlessConfig> {
    // Count occurrences of each name
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for cfg in &configs {
        *name_counts.entry(cfg.name.clone()).or_insert(0) += 1;
    }

    // Find names that appear more than once
    let duplicate_names: std::collections::HashSet<String> = name_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name)
        .collect();

    // Track seen name+address combinations to ensure uniqueness
    let mut seen: HashMap<String, usize> = HashMap::new();

    // Rename duplicates by appending hostname
    for cfg in &mut configs {
        if duplicate_names.contains(&cfg.name) {
            // Create a new name with hostname suffix
            let new_name = format!("{}-{}", cfg.name, cfg.address);

            // Handle cases where even name+hostname might collide (e.g., same proxy from same source)
            let count = seen.entry(new_name.clone()).or_insert(0);
            if *count > 0 {
                cfg.name = format!("{}-{}", new_name, count);
            } else {
                cfg.name = new_name;
            }
            *seen.get_mut(&cfg.name).unwrap_or(&mut 0) += 1;

            // Re-track with the actual final name
            *seen.entry(cfg.name.clone()).or_insert(0) = 1;
        }
    }

    configs
}
