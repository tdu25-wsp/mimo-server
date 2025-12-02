use axum::{
    Router,
    Json,
    routing::post,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
// authモジュールから必要な関数と型をインポート
use crate::auth::{self, Role};

// ログインリクエストの型定義
// フロントエンドからのリクエスト形式（user_idを使用）
#[derive(Deserialize)]
struct LoginRequest {
    user_id: String,
    password: String,
}

// ログインハンドラ
async fn login(Json(payload): Json<LoginRequest>) -> Response {
    // TODO: ここでDB等を確認してパスワード認証を行う
    if payload.user_id != "admin" || payload.password != "password" {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    // 鍵の読み込み
    // issue_access_token関数は内部でリフレッシュトークンを検証するため、
    // 署名鍵(EncodingKey)と検証鍵(DecodingKey)の両方が必要です。
    // ※ 実際の運用ではAppStateなどで保持し、都度読み込みは避けてください
    let private_path = std::path::Path::new("private_key.pem");
    let public_path = std::path::Path::new("public_key.pem"); // 検証用の公開鍵

    let enc_key = match auth::load_signing_key(private_path) {
        Ok(k) => k,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Signing key error: {}", e)).into_response(),
    };

    let dec_key = match auth::load_verifying_key(public_path) {
        Ok(k) => k,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Verifying key error: {}", e)).into_response(),
    };

    // 1. リフレッシュトークンの発行
    // ハンドラのLoginRequestをauthモジュールのLoginRequestに変換
    let auth_req = auth::LoginRequest {
        username: payload.user_id.clone(),
        password: payload.password.clone(),
    };

    let refresh_token = match auth::issue_refresh_token(&auth_req, &enc_key) {
        Ok(token) => token,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 2. アクセストークンの発行
    // ログイン直後のため、先ほど発行したリフレッシュトークンを使ってアクセストークンを発行します
    // ※ ここでは例として ViewMemo 権限を付与しています
    let access_token = match auth::issue_access_token(
        &payload.user_id,
        Role::ViewMemo,
        &refresh_token,
        &enc_key,
        &dec_key
    ) {
        Ok(token) => token,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 両方のトークンをJSONで返す
    Json(json!({
        "access_token": access_token,
        "refresh_token": refresh_token
    })).into_response()
}

pub fn create_auth_router() -> Router {
    Router::new()
        .route("/login", post(login))
}