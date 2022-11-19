use rand::Rng;
use tokio::sync::Mutex;

use crate::{Config, TwitterConfig};

use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct SharedState {
    pub config: Config,
    pub twitter: Option<TwitterState>,
}

impl SharedState {
    pub async fn new(config: Config) -> Arc<Self> {
        let twitter = config.twitter.as_ref().map(TwitterState::new);

        Arc::new(Self { config, twitter })
    }

    pub fn random_string(&self, len: usize) -> String {
        let mut result = String::new();
        let mut rng = rand::thread_rng();

        for _ in 0..len {
            let mut num: u32 = rng.gen_range(0..62);
            if num < 10 {
                num += 0x30;
            } else if num < 36 {
                num += 0x41 - 10;
            } else {
                num += 0x61 - 36;
            }

            #[allow(clippy::transmute_int_to_char)]
            result.push(unsafe { std::mem::transmute(num) });
        }

        result
    }
}

pub struct TwitterState {
    pub basic_auth: String,
    wait: Mutex<HashMap<String, (String, tokio::task::JoinHandle<()>)>>,
}

impl TwitterState {
    fn new(config: &TwitterConfig) -> Self {
        let mut basic_auth = format!("{}:{}", config.client_id, config.client_secret);
        basic_auth = base64::encode(&basic_auth);

        Self {
            basic_auth,
            wait: Mutex::const_new(HashMap::new()),
        }
    }

    pub async fn wait_add(
        &self,
        state: &Arc<SharedState>,
        state_id: String,
        code_verifier: String,
    ) {
        let state = Arc::clone(state);
        let state_id2 = state_id.clone();
        let handle = tokio::spawn(async move {
            let state_id = state_id2;
            tokio::time::sleep(Duration::from_secs(3 * 10)).await;

            state
                .twitter
                .as_ref()
                .unwrap()
                .wait
                .lock()
                .await
                .remove(&state_id);
        });

        self.wait
            .lock()
            .await
            .insert(state_id, (code_verifier, handle));
    }

    pub async fn wait_get(&self, state_id: String) -> Option<String> {
        let wait = self.wait.lock().await.remove(&state_id)?;
        wait.1.abort();

        Some(wait.0)
    }
}
