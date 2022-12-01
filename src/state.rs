use axum::response;
use chrono::Utc;
use oauth2::{
    AuthType, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeVerifier, RedirectUrl,
    RevocationUrl, TokenUrl,
};
use sha3::{Digest, Sha3_512};
use sqlx::MySqlPool;
use tokio::sync::Mutex;

use crate::Config;

use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct SharedState {
    pub config: Config,
    pub db: MySqlWrapper,
    pub oauth: OauthState,
}

impl SharedState {
    pub async fn new(config: Config) -> Arc<Self> {
        let db = MySqlWrapper::new(&config.mysql).await;
        let oauth = OauthState::new(&config);

        Arc::new(Self { config, db, oauth })
    }
}

pub struct MySqlWrapper(MySqlPool);

impl MySqlWrapper {
    async fn new(url: &str) -> Self {
        let mysql = Self(MySqlPool::connect(url).await.unwrap());
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS
            account_session (
                id TEXT CHARACTER SET utf8mb4 NOT NULL,
                token_hash BLOB NOT NULL,
                time BIGINT UNSIGNED NOT NULL,
                PRIMARY KEY(id(384), token(384))
            );",
        )
        .execute(&mysql.0)
        .await
        .unwrap();

        mysql
    }

    pub async fn new_account_session(&self, id: &str) -> Option<response::Response> {
        let token = rand::thread_rng().gen_alphanumeric(86);

        let mut token_hashed = Sha3_512::new();
        token_hashed.update(&token);
        let token_hashed = token_hashed.finalize();

        let time = Utc::now().timestamp();
        if time.is_negative() {
            return None;
        }
        let time = time as u64;

        sqlx::query(
            "INSERT INTO
                account_session (
                    id, token_hash, time
                )
                VALUE (
                    ?, ?, ?
                );",
        )
        .bind(id)
        .bind(token_hashed.as_slice())
        .bind(time)
        .execute(&self.0)
        .await
        .ok()?;

        None
    }
}

pub struct OauthState {
    pub twitter: Option<oauth2::basic::BasicClient>,
    pub google: Option<oauth2::basic::BasicClient>,
    wait: Mutex<HashMap<String, (PkceCodeVerifier, tokio::task::JoinHandle<()>)>>,
}

impl OauthState {
    fn new(config: &Config) -> Self {
        let mut twitter_auth = None;
        if let Some(twitter) = &config.twitter {
            let twitter = oauth2::basic::BasicClient::new(
                ClientId::new(twitter.client_id.clone()),
                Some(ClientSecret::new(twitter.client_secret.clone())),
                AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string()).unwrap(),
                Some(TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string()).unwrap()),
            )
            .set_auth_type(AuthType::BasicAuth)
            .set_revocation_uri(
                RevocationUrl::new("https://api.twitter.com/2/oauth2/revoke".to_string()).unwrap(),
            )
            .set_redirect_uri(
                RedirectUrl::new(format!("https://{}/api/account/twitter_auth", &config.host))
                    .unwrap(),
            );

            twitter_auth = Some(twitter);
        }

        let mut google_auth = None;
        if let Some(google) = &config.google {
            let google = oauth2::basic::BasicClient::new(
                ClientId::new(google.client_id.clone()),
                Some(ClientSecret::new(google.client_secret.clone())),
                AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
                Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
            )
            .set_auth_type(AuthType::BasicAuth)
            .set_revocation_uri(
                RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string()).unwrap(),
            )
            .set_redirect_uri(
                RedirectUrl::new(format!("https://{}/api/account/google_auth", &config.host))
                    .unwrap(),
            );

            google_auth = Some(google);
        }

        Self {
            twitter: twitter_auth,
            google: google_auth,
            wait: Mutex::const_new(HashMap::new()),
        }
    }

    pub async fn wait_add(
        &self,
        state: &Arc<SharedState>,
        state_id: CsrfToken,
        code_verifier: PkceCodeVerifier,
    ) {
        let state = Arc::clone(state);
        let state_id2 = state_id.clone();
        let handle = tokio::spawn(async move {
            let state_id = state_id2;
            tokio::time::sleep(Duration::from_secs(3 * 10)).await;

            state.oauth.wait.lock().await.remove(state_id.secret());
        });

        self.wait
            .lock()
            .await
            .insert(state_id.secret().clone(), (code_verifier, handle));
    }

    pub async fn wait_get(&self, state_id: String) -> Option<PkceCodeVerifier> {
        let wait = self.wait.lock().await.remove(&state_id)?;
        wait.1.abort();

        Some(wait.0)
    }
}

#[allow(clippy::transmute_int_to_char)]
trait RandExt: rand::Rng {
    fn gen_alphanumeric(&mut self, len: usize) -> String {
        let mut result = String::with_capacity(len);

        for _ in 0..len {
            let charactor = self.gen_range(0..62);
            if charactor < 10 {
                result.push(unsafe { std::mem::transmute(0x30 + charactor) });
            } else if charactor < 36 {
                result.push(unsafe { std::mem::transmute(0x41 - 10 + charactor) });
            } else {
                result.push(unsafe { std::mem::transmute(0x61 - 36 + charactor) });
            }
        }

        result
    }
}
impl<T> RandExt for T where T: rand::Rng {}
