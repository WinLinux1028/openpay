use std::sync::Arc;

use axum::{
    extract,
    http::StatusCode,
    response::{self, IntoResponse, Response},
};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope, TokenResponse};

use crate::{
    api::{account::OauthQuery, internal_server_error, no_cache},
    state::SharedState,
};

pub async fn google_login(extract::State(state): extract::State<Arc<SharedState>>) -> Response {
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

    (no_cache(), response::Redirect::temporary(auth_url.as_str())).into_response()
}

pub async fn google_auth(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Query(oauth): extract::Query<OauthQuery>,
) -> response::Response {
    let google = match &state.oauth.google {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Google is disabled").into_response(),
    };

    let code_verifier = match state.oauth.wait_get(oauth.state).await {
        Some(s) => s,
        None => return internal_server_error(),
    };
    let token = google
        .exchange_code(AuthorizationCode::new(oauth.code))
        .set_pkce_verifier(code_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await;
    let token = match token {
        Ok(o) => o,
        Err(_) => return internal_server_error(),
    };
    let token = token.access_token().secret();

    // ユーザーIDを取得する
    let client = reqwest::Client::new();

    let user_id = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .header("Authorization", format!("Bearer {}", &token))
        .send()
        .await;
    let user_id = match user_id {
        Ok(o) => o,
        Err(_) => return internal_server_error(),
    };

    let user_id: GoogleID = match user_id.json().await {
        Ok(o) => o,
        Err(_) => return internal_server_error(),
    };

    if !user_id.verified_email {
        return (
            StatusCode::FORBIDDEN,
            no_cache(),
            "Verify your email address",
        )
            .into_response();
    }
    let user_id = user_id.id;

    (no_cache(), user_id).into_response()
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
