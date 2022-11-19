use std::sync::Arc;

use axum::{
    body, extract,
    http::StatusCode,
    response::{self, IntoResponse, Response},
};
use sha2::Digest;

use crate::state::SharedState;

pub async fn twitter_login(extract::State(state): extract::State<Arc<SharedState>>) -> Response {
    let twitter = match &state.twitter {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Twitter is disabled").into_response(),
    };

    let state_id = state.random_string(500);

    let code_verifier = state.random_string(43);
    let code_verifier = base64::encode_config(code_verifier, base64::URL_SAFE_NO_PAD);

    twitter
        .wait_add(&state, state_id.clone(), code_verifier.clone())
        .await;

    let mut code_challenge = sha2::Sha256::new();
    code_challenge.update(code_verifier);
    let code_challenge = code_challenge.finalize();
    let code_challenge = base64::encode_config(code_challenge, base64::URL_SAFE_NO_PAD);

    response::Redirect::temporary(
        &format!("https://twitter.com/i/oauth2/authorize?response_type=code&code_challenge_method=s256&scope=offline.access%20tweet.read%20users.read&client_id={}&redirect_uri=https://{}/api/account/twitter_auth&state={}&code_challenge={}",
            &state.config.twitter.as_ref().unwrap().client_id,
            &state.config.host,
            state_id,
            code_challenge
        )
    ).into_response()
}

pub async fn twitter_auth(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Query(oauth): extract::Query<Twitter>,
) -> response::Response {
    let twitter = match &state.twitter {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Twitter is disabled").into_response(),
    };
    let code_verifier = match twitter.wait_get(oauth.state).await {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Invalid state").into_response(),
    };

    format!(
        "curl --location --request POST 'https://api.twitter.com/2/oauth2/token' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'grant_type=authorization_code' \
--header 'Authorization: Basic {}' \
--data-urlencode 'redirect_uri=https://{}/api/account/twitter_auth' \
--data-urlencode 'code={}' \
--data-urlencode 'code_verifier={}'",
        &twitter.basic_auth, &state.config.host, &oauth.code, code_verifier
    )
    .into_response()
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Twitter {
    state: String,
    code: String,
}
