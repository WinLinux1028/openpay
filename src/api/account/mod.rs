mod handler;

use crate::{api::account::handler::*, api::method, state::SharedState};

use axum::{body, routing::Router};
use std::sync::Arc;

pub async fn router<B>() -> Router<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    Router::new()
        .route("/twitter_login", method().get(twitter_login))
        .route("/twitter_auth", method().get(twitter_auth))
}
