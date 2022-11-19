use oauth2::{
    AuthType, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeVerifier, RedirectUrl,
    RevocationUrl, TokenUrl,
};
use tokio::sync::Mutex;

use crate::Config;

use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct SharedState {
    pub config: Config,
    pub oauth: OauthState,
}

impl SharedState {
    pub async fn new(config: Config) -> Arc<Self> {
        let oauth = OauthState::new(&config);

        Arc::new(Self { config, oauth })
    }
}

pub struct OauthState {
    pub twitter: Option<oauth2::basic::BasicClient>,
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

        Self {
            twitter: twitter_auth,
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
