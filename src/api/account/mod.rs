mod handler_go;
mod handler_tw;

use crate::{
    api::account::{handler_go::*, handler_tw::*},
    api::method,
    state::SharedState,
};

use axum::{body, routing::Router};
use std::sync::Arc;

pub async fn router<B>() -> Router<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    Router::new()
        .route("/twitter_login", method().get(twitter_login))
        .route("/twitter_auth", method().get(twitter_auth))
        .route("/google_login", method().get(google_login))
        .route("/google_auth", method().get(google_auth))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OauthQuery {
    state: String,
    code: String,
}
