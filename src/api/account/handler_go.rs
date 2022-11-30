use std::sync::Arc;

use axum::{
    extract,
    http::StatusCode,
    response::{self, IntoResponse},
};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope, TokenResponse};

use crate::{
    api::{account::OauthQuery, status_500, ErrWrapper, NoCache},
    state::SharedState,
};

pub async fn google_login(
    extract::State(state): extract::State<Arc<SharedState>>,
) -> response::Response {
    let google = match &state.oauth.google {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Google is disabled").into_response(),
    };

    let (code_challenge, code_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, state_id) = google
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .add_scope(Scope::new("openid".to_string()))
        .set_pkce_challenge(code_challenge)
        .url();

    state.oauth.wait_add(&state, state_id, code_verifier).await;

    response::Redirect::temporary(auth_url.as_str()).no_cache()
}

pub async fn google_auth(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Query(oauth): extract::Query<OauthQuery>,
) -> Result<response::Response, ErrWrapper> {
    let google = match &state.oauth.google {
        Some(s) => s,
        None => return Ok((StatusCode::NOT_FOUND, "Google is disabled").into_response()),
    };

    let code_verifier = match state.oauth.wait_get(oauth.state).await {
        Some(s) => s,
        None => return Ok(status_500().no_cache()),
    };
    let token = google
        .exchange_code(AuthorizationCode::new(oauth.code))
        .set_pkce_verifier(code_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await?;
    let token = token.access_token().secret();

    // ユーザーIDを取得する
    let client = reqwest::Client::new();

    let user_id = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .header("Authorization", format!("Bearer {}", &token))
        .send()
        .await?
        .text()
        .await?;

    let user_id: GoogleID = serde_json::from_str(&user_id)?;
    if !user_id.verified_email {
        return Ok((StatusCode::FORBIDDEN, "Verify your email address").no_cache());
    }
    let user_id = user_id.id;

    Ok(user_id.no_cache())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GoogleID {
    id: String,
    email: String,
    verified_email: bool,
    name: String,
    given_name: String,
    picture: String,
    locale: String,
}
