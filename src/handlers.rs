use crate::config::Config;
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
    let target_url = match params.get("url") {
        Some(url) => url,
        None => return (StatusCode::BAD_REQUEST, "Missing 'url' query parameter").into_response(),
    };

    println!("Fetching subscription from: {}", target_url);

    let client = reqwest::Client::new();
    let resp = match client
        .get(target_url)
        .header(USER_AGENT, &state.config.subscription_user_agent)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch subscription: {}", e),
            )
                .into_response()
        }
    };

    if !resp.status().is_success() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch subscription: {}", resp.status()),
        )
            .into_response();
    }

    let encoded_body = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read response body: {}", e),
            )
                .into_response()
        }
    };

    // Decode Base64
    let decoded_body = match decode_base64_content(&encoded_body) {
        Ok(d) => d,
        Err(e) => {
             return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to decode base64 subscription body: {}", e),
            )
            .into_response()
        }
    };

    let links: Vec<&str> = decoded_body
        .split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    let mut vless_configs = Vec::new();

    for link in links {
        if let Ok(config) = parse_vless_link(link) {
            let is_filtered = state.config.filter_keywords.iter().any(|w| config.name.contains(w));
            if is_filtered {
                println!("Filtered out proxy: {}", config.name);
                continue;
            }
            vless_configs.push(config);
        }
    }

    if vless_configs.is_empty() {
        return (StatusCode::NOT_FOUND, "No valid VLESS links found").into_response();
    }

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
        .into_response()
    }

    println!("Written Xray config to {}", state.config.config_path);

    let surge_list = generate_surge_list(&vless_configs, &state.config);

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        surge_list,
    )
    .into_response()
}

fn decode_base64_content(content: &str) -> anyhow::Result<String> {
    let trimmed = content.trim();
    // Sometimes padding might be missing or whitespace issues
    let decoded_bytes = general_purpose::STANDARD.decode(trimmed)?;
    String::from_utf8(decoded_bytes).map_err(|e| anyhow::anyhow!(e))
}
