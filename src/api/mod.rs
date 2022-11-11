use crate::{
    state::{ArcSharedState, SharedState},
    Config,
};

use std::sync::Arc;

use axum::{body, extract, response, routing::Router};

pub async fn router<B>(config: Config) -> Router<Arc<SharedState>, B>
where
    B: body::HttpBody + Send + 'static,
{
    let state = SharedState::new(config).await;

    state
        .router()
        .route("/a", state.method().get(hello_world))
        .route("/b", state.method().get(hello_world2))
}

async fn hello_world(
    extract::State(state): extract::State<Arc<SharedState>>,
) -> response::Html<String> {
    response::Html(format!("Hello World!, from {}", &state.config.host))
}

async fn hello_world2() -> &'static str {
    "Hello World!"
}
