mod account;

use crate::state::SharedState;

use axum::{
    body,
    http::{HeaderMap, StatusCode},
    response::{self, IntoResponse},
    routing::{MethodRouter, Router},
};
use std::sync::Arc;

pub async fn router<B>() -> Router<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    Router::new()
        .fallback(not_found)
        .nest("/account", account::router().await)
}

pub fn method<B>() -> MethodRouter<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    MethodRouter::new().fallback(method_not_allowed)
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 Not Found")
}
async fn method_not_allowed() -> (StatusCode, &'static str) {
    (StatusCode::METHOD_NOT_ALLOWED, "405 Method Not Allowed")
}

fn internal_server_error() -> response::Response {
    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", "no-store".parse().unwrap());

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        headers,
        "500 Internal Server Error",
    )
        .into_response()
}
