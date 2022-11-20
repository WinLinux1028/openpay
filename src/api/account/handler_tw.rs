use std::sync::Arc;

use axum::{
    extract,
    http::StatusCode,
    response::{self, IntoResponse, Response},
};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope, TokenResponse};

use crate::{
    api::{account::OauthQuery, status_500, ErrWrapper, NoCache},
    state::SharedState,
};

pub async fn twitter_login(extract::State(state): extract::State<Arc<SharedState>>) -> Response {
    let twitter = match &state.oauth.twitter {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Twitter is disabled").into_response(),
    };

    let (code_challenge, code_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, state_id) = twitter
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("tweet.read".to_string()))
        .add_scope(Scope::new("users.read".to_string()))
        .set_pkce_challenge(code_challenge)
        .url();

    state.oauth.wait_add(&state, state_id, code_verifier).await;

    NoCache(response::Redirect::temporary(auth_url.as_str())).into_response()
}

pub async fn twitter_auth(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Query(oauth): extract::Query<OauthQuery>,
) -> Result<response::Response, ErrWrapper> {
    let twitter = match &state.oauth.twitter {
        Some(s) => s,
        None => return Ok((StatusCode::NOT_FOUND, "Twitter is disabled").into_response()),
    };

    let code_verifier = match state.oauth.wait_get(oauth.state).await {
        Some(s) => s,
        None => return Ok(NoCache(status_500()).into_response()),
    };

    let token = twitter
        .exchange_code(AuthorizationCode::new(oauth.code))
        .set_pkce_verifier(code_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await?;
    let token = token.access_token().secret();

    // ユーザーIDを取得する
    let client = reqwest::Client::new();

    let user_id = client
        .get("https://api.twitter.com/2/users/me")
        .header("Authorization", format!("Bearer {}", &token))
        .send()
        .await?
        .text()
        .await?;

    let user_id: TwitterData<TwitterID> = serde_json::from_str(&user_id)?;
    let user_id = user_id.data.id;

    Ok(NoCache(user_id).into_response())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TwitterData<T> {
    data: T,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TwitterID {
    id: String,
    name: String,
    username: String,
}
