mod config;
mod vless;
mod xray;
mod surge;
mod handlers; // We will create this next

use crate::config::Config;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct AppState {
    config: Config,
}

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize config
    let config = Config::from_env();
    let port = config.server_port;
    
    let state = Arc::new(AppState { config });

    // Build router
    let app = Router::new()
        .route("/", get(handlers::fetch_subscription))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on http://{}", addr);
    println!("Xray Config will be saved to: {}", Config::from_env().config_path); // creating new config here just for logging path

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
