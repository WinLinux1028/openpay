use crate::Config;

use std::sync::Arc;

pub struct SharedState {
    pub config: Config,
}

impl SharedState {
    pub async fn new(config: Config) -> Arc<Self> {
        Arc::new(SharedState { config })
    }
}
