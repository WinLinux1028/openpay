use std::{collections::HashMap, sync::Arc};

use axum::{
    extract,
    http::StatusCode,
    response::{self, IntoResponse, Response},
};
use sha2::Digest;

use crate::{
    api::{internal_server_error, no_cache},
    state::SharedState,
};

pub async fn twitter_login(extract::State(state): extract::State<Arc<SharedState>>) -> Response {
    let twitter = match &state.twitter {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Twitter is disabled").into_response(),
    };

    // ログイン毎にユニークな値を作成
    let state_id = state.random_string(500);
    let code_verifier = state.random_string(43);
    let code_verifier = base64::encode_config(code_verifier, base64::URL_SAFE_NO_PAD);

    // 後で使うためstateとcode_verifierの対を保存しておく
    twitter
        .wait_add(&state, state_id.clone(), code_verifier.clone())
        .await;

    // リダイレクトURLの作成に必要な情報を作る
    let mut code_challenge = sha2::Sha256::new();
    code_challenge.update(code_verifier);
    let code_challenge = code_challenge.finalize();
    let code_challenge = base64::encode_config(code_challenge, base64::URL_SAFE_NO_PAD);

    (
        no_cache(),
        response::Redirect::temporary(
            &format!("https://twitter.com/i/oauth2/authorize?response_type=code&code_challenge_method=s256&scope=tweet.read%20users.read&client_id={}&redirect_uri=https://{}/api/account/twitter_auth&state={}&code_challenge={}", 
                &state.config.twitter.as_ref().unwrap().client_id,
                &state.config.host,
                state_id,
                code_challenge
            )
        )
    ).into_response()
}

pub async fn twitter_auth(
    extract::State(state): extract::State<Arc<SharedState>>,
    extract::Query(oauth): extract::Query<TwitterAuth>,
) -> response::Response {
    let twitter = match &state.twitter {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "Twitter is disabled").into_response(),
    };
    // 事前に保存しておいたcode_verifierをstateで取得
    let code_verifier = match twitter.wait_get(oauth.state).await {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, no_cache(), "Invalid state").into_response(),
    };

    // アカウントを操作するためのtokenを取得
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("grant_type", String::from("authorization_code"));
    params.insert(
        "redirect_uri",
        format!("https://{}/api/account/twitter_auth", &state.config.host),
    );
    params.insert("code", oauth.code.clone());
    params.insert("code_verifier", code_verifier);

    let token = client
        .post("https://api.twitter.com/2/oauth2/token")
        .header("Authorization", format!("Basic {}", &twitter.basic_auth))
        .form(&params)
        .send()
        .await;
    let token = match token {
        Ok(o) => o,
        Err(_) => return internal_server_error(),
    };

    let token: TwitterToken = match token.json().await {
        Ok(o) => o,
        Err(_) => return internal_server_error(),
    };
    let token = token.access_token;

    // ユーザーIDを取得する
    let user_id = client
        .get("https://api.twitter.com/2/users/me")
        .header("Authorization", format!("Bearer {}", &token))
        .send()
        .await;
    let user_id = match user_id {
        Ok(o) => o,
        Err(_) => return internal_server_error(),
    };

    let user_id: TwitterData<TwitterID> = match user_id.json().await {
        Ok(o) => o,
        Err(_) => return internal_server_error(),
    };
    let user_id = user_id.data.id;

    (no_cache(), user_id).into_response()
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TwitterAuth {
    state: String,
    code: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TwitterToken {
    access_token: String,
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
