use crate::Config;

use std::sync::Arc;

use axum::{
    body,
    http::StatusCode,
    routing::{MethodRouter, Router},
};

pub struct SharedState {
    pub config: Config,
}

impl SharedState {
    pub async fn new(config: Config) -> Arc<Self> {
        Arc::new(SharedState { config })
    }
}

pub trait ArcSharedState<B>
where
    B: body::HttpBody + Send + 'static,
{
    fn router(&self) -> Router<Arc<SharedState>, B>;
    fn method(&self) -> MethodRouter<Arc<SharedState>, B>;
}

impl<B> ArcSharedState<B> for Arc<SharedState>
where
    B: body::HttpBody + Send + 'static,
{
    fn router(&self) -> Router<Arc<SharedState>, B> {
        Router::with_state(Arc::clone(self)).fallback(not_found)
    }

    fn method(&self) -> MethodRouter<Arc<SharedState>, B> {
        MethodRouter::new().fallback(method_not_allowed)
    }
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

async fn method_not_allowed() -> (StatusCode, &'static str) {
    (StatusCode::METHOD_NOT_ALLOWED, "405 Method Not Allowed")
}
