mod api;
mod state;

use std::sync::Arc;

use axum::{http::StatusCode, routing::Router};
use tokio::{fs, io::AsyncReadExt};

use crate::state::SharedState;

#[tokio::main]
async fn main() {
    let mut config = String::new();
    fs::File::open("./config.json")
        .await
        .unwrap()
        .read_to_string(&mut config)
        .await
        .unwrap();
    let config: Config = serde_json::from_str(&config).unwrap();
    let state = SharedState::new(config).await;

    let root = Router::new()
        .nest("/api/", api::router().await)
        .fallback(unusual_access)
        .with_state(Arc::clone(&state));

    axum::Server::bind(&state.config.listen)
        .tcp_nodelay(true)
        .serve(root.into_make_service())
        .await
        .unwrap();
}

async fn unusual_access() -> (StatusCode, &'static str) {
    (
        StatusCode::FORBIDDEN,
        "APIサーバーに来ないはずのパスへのアクセスが発生しました\nリバースプロキシの設定を確認してください",
    )
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    listen: std::net::SocketAddr,
    host: String,
    twitter: Option<TwitterConfig>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TwitterConfig {
    client_id: String,
    client_secret: String,
}
