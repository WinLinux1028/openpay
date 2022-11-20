mod account;

use crate::state::SharedState;

use axum::{
    body,
    http::StatusCode,
    response::{self, IntoResponse},
    routing::{MethodRouter, Router},
};
use std::sync::Arc;

pub async fn router<B>() -> Router<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    Router::new()
        .fallback(|| async { status_404() })
        .nest("/account", account::router().await)
}

pub fn method<B>() -> MethodRouter<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    MethodRouter::new().fallback(|| async { status_405() })
}

fn status_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 Not Found")
}
fn status_405() -> (StatusCode, &'static str) {
    (StatusCode::METHOD_NOT_ALLOWED, "405 Method Not Allowed")
}

fn status_500() -> (StatusCode, &'static str) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "500 Internal Server Error",
    )
}

struct NoCache<T: IntoResponse>(T);

impl<T> IntoResponse for NoCache<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> response::Response {
        let mut response = self.0.into_response();
        response
            .headers_mut()
            .insert("Cache-Control", "no-store".parse().unwrap());

        response
    }
}
