use std::sync::Arc;

use axum::extract;

use crate::state::SharedState;

pub async fn twitter_login(extract::State(state): extract::State<Arc<SharedState>>) {}

pub async fn twitter_auth(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Query(oauth): extract::Query<Twitter>,
) -> &'static str {
    "Hello World!"
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Twitter {
    state: String,
    code: String,
}
