use axum::{
    Router,
    routing::{get, post},
    response::{IntoResponse, Json},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::json;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

pub fn create_auth_routes() -> Router {
    Router::new()
        .route("/auth/login", post(handle_login))
        .route("/auth/logout", post(handle_logout))
        .route("/auth/me", get(handle_get_current_user))
        .route("/auth/register", post(handle_register))
        .route("/auth/refresh", post(issue_access_token))
        .route("/auth/reset-password", post(handle_reset_password))
        .route("/auth/forgot-password", post(handle_forgot_password))
        .route("/auth/verify", post(handle_verification_code))
        .route("/auth/verify-email", post(handle_verify_email))
}

//// ログインハンドラ
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

// ログイン要求を検証し、問題無い場合にリフレッシュトークンを含めて返却する
async fn handle_login(jar: CookieJar) -> impl IntoResponse  {
    // Dummy
    let token = "1234.1244.2141";

    // Cookieの構築
    let cookie = Cookie::build(("refresh_token", token))
        .path("/api/auth/refresh")
        .http_only(true)
        .secure(false) //開発中なのでhttpを許可
        .same_site(SameSite::Lax)
        .build();

    (
        jar.add(cookie), 
        Json(json!({"message": "Login successful", "refresh_token": token}))
    )

}

async fn handle_logout(jar: CookieJar) -> impl IntoResponse {
    // Cookieからトークンを回収した後に削除する
    let refresh_token = jar.get("refresh_token");
    let access_token = jar.get("access_token");
    jar.remove(refresh_token.unwrap());
    jar.remove(access_token.unwrap());

    // TODO: Revoke tokens

    (jar, Json(json!({"message": "Logout successful"})))
}

async fn handle_get_current_user(jar: CookieJar) -> impl IntoResponse {
    // TODO: DBへの問い合わせ
    let token = jar.get("access_token");
    if token.is_none() {
        return (jar, Json(json!({"error": "Unauthorized"})));
    }

    (Json(json!({"DisplayName": "John Doe"})))
}

async fn handle_refresh(jar: CookieJar) -> impl IntoResponse {
    let refresh_token = jar.get("refresh_token");
    if refresh_token.is_none() {
        return (jar, Json(json!({"error": "Unauthorized"})));
    }

    // TODO: Validate Token
 

    // TODO: Issue valid access token
    access_token = "new_access"
}

//// ユーザ登録ハンドラ
#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
}

async fn handle_register(req: Json<RegisterRequest>, jar: CookieJar) -> impl IntoResponse {
    let register_token = jar.get("register_token");
    if register_token.is_none() {
        return (jar, Json(json!({"error": "Unauthorized! Please start registration again"}
    )));



}

async fn issue_access_token(jar: CookieJar) -> impl IntoResponse {
    let refresh_token = jar.get("refresh_token");
    if refresh_token.is_none() {
        return (jar, Json(json!({"error": "Unauthorized! Please login again"})));
    }
}

async fn handle_reset_password() {
    // Implementation here
    todo!()
}

async fn handle_forgot_password() {
    // Implementation here
    todo!()
}

async fn handle_verification_code() {
    // Implementation here
    todo!()
}

async fn handle_verify_email() {
    // Implementation here
    todo!()
}
