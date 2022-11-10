mod api;

use axum::{http::StatusCode, routing::Router};
use tokio::{fs, io::AsyncReadExt};

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

    let root = Router::new()
        .fallback(unusual_access)
        .nest("/api/", api::router(config.clone()).await);

    axum::Server::bind(&config.listen)
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    listen: std::net::SocketAddr,
    host: String,
}
