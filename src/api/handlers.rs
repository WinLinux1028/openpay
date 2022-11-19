use crate::state::SharedState;

use axum::{extract, response};
use std::sync::Arc;

pub async fn hello_world(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Path(name): extract::Path<String>,
) -> response::Html<String> {
    response::Html(format!("Hello {}, from {}", name, &state.config.host))
}

pub async fn hello_world2(extract::Path(id): extract::Path<u64>) -> String {
    format!("Hello {}", id)
}
