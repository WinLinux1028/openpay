use std::{sync::Arc, time::Duration};

use axum::extract;
use sha2::Digest;

use crate::state::SharedState;

pub async fn twitter_login(extract::State(state): extract::State<Arc<SharedState>>) -> String {
    let twitter = match &state.twitter {
        Some(s) => s,
        None => return String::from("err"),
    };

    let state_id = state.random_string(500);

    let code_verifier = state.random_string(43);
    let code_verifier = base64::encode_config(code_verifier, base64::URL_SAFE_NO_PAD);

    let state2 = Arc::clone(&state);
    let code_verifier2 = code_verifier.clone();
    let handle = tokio::spawn(async move {
        let state = state2;
        let code_verifier = code_verifier2;
        tokio::time::sleep(Duration::from_secs(3 * 60)).await;

        state
            .twitter
            .as_ref()
            .unwrap()
            .wait
            .lock()
            .await
            .remove(&code_verifier);
    });
    twitter
        .wait
        .lock()
        .await
        .insert(state_id.clone(), (code_verifier.clone(), handle));

    let mut code_challenge = sha2::Sha256::new();
    code_challenge.update(code_verifier);
    let code_challenge = code_challenge.finalize();
    let code_challenge = base64::encode_config(code_challenge, base64::URL_SAFE_NO_PAD);

    format!("https://twitter.com/i/oauth2/authorize?response_type=code&code_challenge_method=s256&scope=offline.access%20tweet.read%20users.read&client_id={}&redirect_uri=https://{}/api/account/twitter_auth&state={}&code_challenge={}",
        &state.config.twitter.as_ref().unwrap().client_id,
        &state.config.host,
        state_id,
        code_challenge
    )
}

pub async fn twitter_auth(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Query(oauth): extract::Query<Twitter>,
) -> String {
    let twitter = match &state.twitter {
        Some(s) => s,
        None => return String::from("err"),
    };

    let wait = match twitter.wait.lock().await.remove(&oauth.state) {
        Some(s) => s,
        None => return String::from("err"),
    };
    wait.1.abort();

    format!(
        "curl --location --request POST 'https://api.twitter.com/2/oauth2/token' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'grant_type=authorization_code' \
--header 'Authorization: Basic {}' \
--data-urlencode 'redirect_uri=https://{}/api/account/twitter_auth' \
--data-urlencode 'code={}' \
--data-urlencode 'code_verifier={}'",
        &twitter.basic_auth, &state.config.host, &oauth.code, wait.0
    )
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Twitter {
    state: String,
    code: String,
}
