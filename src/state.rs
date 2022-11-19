use crate::Config;

use std::sync::Arc;

pub struct SharedState {
    pub config: Config,
    pub twitter_auth: Option<String>,
}

impl SharedState {
    pub async fn new(config: Config) -> Arc<Self> {
        let mut twitter_auth = None;
        if let Some(twitter) = &config.twitter {
            let twitter = format!("{}:{}", twitter.client_id, twitter.client_secret);
            twitter_auth = Some(base64::encode(twitter));
        }

        Arc::new(SharedState {
            config,
            twitter_auth,
        })
    }
}
