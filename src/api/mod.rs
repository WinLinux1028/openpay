use crate::Config;

use std::sync::Arc;

use axum::{
    body, extract,
    http::StatusCode,
    response,
    routing::{MethodRouter, Router},
};

pub async fn router<B>(config: Config) -> Router<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    let state = SharedState::new(config).await;

    let base = MethodRouter::new().fallback(method_not_allowed);

    Router::with_state(Arc::clone(&state))
        .fallback(not_found)
        .route("/a", base.clone().get(hello_world))
        .route("/b", base.get(hello_world2))
}

async fn hello_world(
    extract::State(state): extract::State<Arc<SharedState>>,
) -> response::Html<String> {
    response::Html(format!("Hello World!, from {}", &state.config.host))
}

async fn hello_world2() -> &'static str {
    "Hello World!"
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

async fn method_not_allowed() -> (StatusCode, &'static str) {
    (StatusCode::METHOD_NOT_ALLOWED, "405 Method Not Allowed")
}

pub struct SharedState {
    config: Config,
}

impl SharedState {
    async fn new(config: Config) -> Arc<SharedState> {
        Arc::new(SharedState { config })
    }
}
