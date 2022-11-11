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
        .route("/a/:name", state.method().get(hello_world))
        .route("/b/:id", state.method().get(hello_world2))
}

async fn hello_world(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Path(name): extract::Path<String>,
) -> response::Html<String> {
    response::Html(format!("Hello {}, from {}", name, &state.config.host))
}

async fn hello_world2(extract::Path(id): extract::Path<u64>) -> String {
    format!("Hello {}", id)
}
