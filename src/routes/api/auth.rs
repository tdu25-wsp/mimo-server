use axum::{
    Router,
    extract::{State, ConnectInfo},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use time;

use crate::auth::{create_decoding_key, extract_jti_from_token, extract_user_id_from_token};
use crate::error::{AppError, map_error};
use crate::repositories::auth::UserCreateRequest;
use crate::server::AppState;

/// クライアントの実際のIPアドレスを取得
/// Cloudflare Tunnel経由の場合はCF-Connecting-IPヘッダーから取得
fn get_client_ip(headers: &HeaderMap, addr: &SocketAddr) -> String {
    // 1. CF-Connecting-IP (Cloudflare推奨)
    if let Some(ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // 2. X-Real-IP
    if let Some(ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // 3. X-Forwarded-For (最初のIPを取得)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    // 4. フォールバック: 直接接続のIP
    addr.ip().to_string()
}

pub fn create_auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(handle_login))
        .route("/auth/logout", post(handle_logout))
        .route("/auth/me", get(handle_get_current_user))
        .route("/auth/register/start", post(handle_start_registration))
        .route("/auth/register/verify", post(handle_verify_email))
        .route(
            "/auth/register/complete",
            post(handle_complete_registration),
        )
        .route("/auth/refresh", post(handle_refresh))
        .route("/auth/reset-password", post(handle_reset_password))
        .route("/auth/password/forgot", post(handle_forgot_password))
        .route("/auth/password/verify", post(handle_verify_reset_code))
        .route("/auth/password/reset", post(handle_complete_password_reset))
}

// リクエスト/レスポンス構造体
#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct StartRegistrationRequest {
    email: String,
}

#[derive(Deserialize)]
struct VerifyEmailRequest {
    email: String,
    code: String,
}

#[derive(Deserialize)]
struct CompleteRegistrationRequest {
    user_id: String,
    email: String,
    display_name: Option<String>,
    password: String,
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    old_password: String,
    new_password: String,
}

#[derive(Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

#[derive(Deserialize)]
struct VerifyResetCodeRequest {
    email: String,
    code: String,
}

#[derive(Deserialize)]
struct CompletePasswordResetRequest {
    email: String,
    new_password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    message: String,
    user: Option<serde_json::Value>,
}

/// ログイン処理
async fn handle_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, Response> {
    // レート制限チェック（IPベース）
    let ip = get_client_ip(&headers, &addr);
    state
        .auth_rate_limiter
        .check_ip_limit(&ip)
        .map_err(|e| {
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": e})),
            )
                .into_response()
        })?;

    // ユーザーIDベースのレート制限
    state
        .auth_rate_limiter
        .check_user_limit(&req.email)
        .map_err(|e| {
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": e})),
            )
                .into_response()
        })?;

    // ログイン処理
    let (access_token, refresh_token, user) = state
        .auth_service
        .login(req.email, req.password)
        .await
        .map_err(map_error)?;

    // Cookieを設定（環境に応じて自動的に設定される）
    let cookie_config = state.config.server.get_cookie_config();

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .max_age(time::Duration::seconds(7 * 24 * 60 * 60))
        .same_site(cookie_config.same_site)
        .secure(cookie_config.secure)
        .http_only(cookie_config.http_only)
        .build();

    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .max_age(time::Duration::seconds(60 * 60))
        .same_site(cookie_config.same_site)
        .secure(cookie_config.secure)
        .http_only(cookie_config.http_only)
        .build();

    let jar = jar.add(refresh_cookie).add(access_cookie);

    Ok((
        jar,
        Json(json!({
            "message": "Login successful",
            "user": {
                "user_id": user.user_id,
                "email": user.email,
                "display_name": user.display_name,
            }
        })),
    ))
}

/// ログアウト処理
async fn handle_logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, Response> {
    let mut jtis = Vec::new();

    // Cookieからトークンを取得してJTIを抽出
    let key = create_decoding_key(&state.auth_service.get_secret());

    if let Some(access_token) = jar.get("access_token") {
        if let Ok(jti) = extract_jti_from_token(access_token.value(), &key) {
            jtis.push(jti);
        }
    }

    if let Some(refresh_token) = jar.get("refresh_token") {
        if let Ok(jti) = extract_jti_from_token(refresh_token.value(), &key) {
            jtis.push(jti);
        }
    }

    // トークンを無効化
    state.auth_service.logout(jtis).await.map_err(map_error)?;

    // Cookieを削除
    let jar = jar
        .remove(Cookie::from("refresh_token"))
        .remove(Cookie::from("access_token"));

    Ok((jar, Json(json!({"message": "Logout successful"}))))
}

/// 現在のユーザー情報取得
async fn handle_get_current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, Response> {
    // アクセストークンを取得
    let token = jar.get("access_token").ok_or_else(|| {
        map_error(AppError::AuthenticationError(
            "Authentication required".to_string(),
        ))
    })?;

    // トークンからユーザーIDとJTIを取得
    let key = create_decoding_key(&state.auth_service.get_secret());
    let user_id = extract_user_id_from_token(token.value(), &key).map_err(map_error)?;
    let jti = extract_jti_from_token(token.value(), &key).map_err(map_error)?;

    // トークンが失効されていないか確認
    if state.auth_service.is_token_revoked(&jti).await.map_err(map_error)? {
        return Err(map_error(AppError::AuthenticationError(
            "Token has been revoked".to_string(),
        )));
    }

    // ユーザー情報を取得
    let user = state
        .auth_service
        .get_current_user(&user_id)
        .await
        .map_err(map_error)?;

    Ok(Json(json!({
        "user": {
            "user_id": user.user_id,
            "email": user.email,
            "display_name": user.display_name,
            "is_active": user.is_active,
        }
    })))
}

/// ステップ1: 登録開始（確認コード送信）
async fn handle_start_registration(
    State(state): State<AppState>,
    Json(req): Json<StartRegistrationRequest>,
) -> Result<impl IntoResponse, Response> {
    // TODO: IPアドレスを取得して渡す
    state
        .auth_service
        .start_registration(req.email, None)
        .await
        .map_err(map_error)?;

    Ok(Json(json!({"message": "Verification code sent to email"})))
}

/// ステップ2: メール検証と登録トークン発行
async fn handle_verify_email(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<impl IntoResponse, Response> {
    let registration_token = state
        .auth_service
        .verify_email_and_issue_registration_token(req.email, req.code)
        .await
        .map_err(map_error)?;

    // 登録トークンをCookieに設定（15分間有効）
    let cookie_config = state.config.server.get_cookie_config();

    let registration_cookie = Cookie::build(("registration_token", registration_token))
        .path("/")
        .max_age(time::Duration::seconds(15 * 60))
        .same_site(cookie_config.same_site)
        .secure(cookie_config.secure)
        .http_only(cookie_config.http_only)
        .build();

    Ok((
        jar.add(registration_cookie),
        Json(json!({"message": "Email verified successfully"})),
    ))
}

/// ステップ3: 登録完了
async fn handle_complete_registration(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<CompleteRegistrationRequest>,
) -> Result<impl IntoResponse, Response> {
    // Cookieから登録トークンを取得
    let registration_token = jar
        .get("registration_token")
        .ok_or_else(|| {
            map_error(AppError::ValidationError(
                "Registration token not found".to_string(),
            ))
        })?
        .value()
        .to_string();

    let user_req = UserCreateRequest {
        user_id: req.user_id,
        email: req.email,
        display_name: req.display_name,
        password: req.password,
    };

    // ユーザー登録
    let (access_token, refresh_token, user) = state
        .auth_service
        .complete_registration(registration_token, user_req)
        .await
        .map_err(map_error)?;

    // Cookieを設定
    let cookie_config = state.config.server.get_cookie_config();

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .max_age(time::Duration::seconds(7 * 24 * 60 * 60))
        .same_site(cookie_config.same_site)
        .secure(cookie_config.secure)
        .http_only(cookie_config.http_only)
        .build();

    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .max_age(time::Duration::seconds(60 * 60))
        .same_site(cookie_config.same_site)
        .secure(cookie_config.secure)
        .http_only(cookie_config.http_only)
        .build();

    // 登録トークンCookieを削除
    let jar = jar
        .remove(Cookie::from("registration_token"))
        .add(refresh_cookie)
        .add(access_cookie);

    Ok((
        jar,
        Json(json!({
            "message": "Registration successful",
            "user": {
                "user_id": user.user_id,
                "email": user.email,
                "display_name": user.display_name,
            }
        })),
    ))
}

/// トークンリフレッシュ
async fn handle_refresh(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<impl IntoResponse, Response> {
    // レート制限チェック（IPベース）
    let ip = get_client_ip(&headers, &addr);
    state
        .auth_rate_limiter
        .check_ip_limit(&ip)
        .map_err(|e| {
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": e})),
            )
                .into_response()
        })?;

    // リフレッシュトークンを取得
    let refresh_token = jar.get("refresh_token").ok_or_else(|| {
        map_error(AppError::AuthenticationError(
            "Refresh token required".to_string(),
        ))
    })?;

    // トークンからユーザーIDとJTIを取得
    let key = create_decoding_key(&state.auth_service.get_secret());
    let user_id = extract_user_id_from_token(refresh_token.value(), &key).map_err(map_error)?;
    let jti = extract_jti_from_token(refresh_token.value(), &key).map_err(map_error)?;

    // トークンが失効されていないか確認
    if state.auth_service.is_token_revoked(&jti).await.map_err(map_error)? {
        return Err(map_error(AppError::AuthenticationError(
            "Token has been revoked".to_string(),
        )));
    }

    // ユーザーIDベースのレート制限
    state
        .auth_rate_limiter
        .check_user_limit(&user_id)
        .map_err(|e| {
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": e})),
            )
                .into_response()
        })?;

    // 新しいアクセストークンを発行
    let access_token = state
        .auth_service
        .refresh_access_token(&user_id)
        .await
        .map_err(map_error)?;

    // 新しいアクセストークンをCookieに設定
    let cookie_config = state.config.server.get_cookie_config();

    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .max_age(time::Duration::seconds(60 * 60))
        .same_site(cookie_config.same_site)
        .secure(cookie_config.secure)
        .http_only(cookie_config.http_only)
        .build();

    Ok((
        jar.add(access_cookie),
        Json(json!({"message": "Token refresh successful"})),
    ))
}

/// パスワードリセット
async fn handle_reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, Response> {
    // アクセストークンからユーザーIDを取得
    let token = jar.get("access_token").ok_or_else(|| {
        map_error(AppError::AuthenticationError(
            "Authentication required".to_string(),
        ))
    })?;

    let key = create_decoding_key(&state.auth_service.get_secret());
    let user_id = extract_user_id_from_token(token.value(), &key).map_err(map_error)?;

    state
        .auth_service
        .reset_password(&user_id, &req.old_password, &req.new_password)
        .await
        .map_err(map_error)?;

    Ok(Json(json!({"message": "Password reset successful"})))
}

/// ステップ1: パスワード忘れ（確認コード送信）
async fn handle_forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, Response> {
    // TODO: IPアドレスを取得して渡す
    state
        .auth_service
        .forgot_password(&req.email, None)
        .await
        .map_err(map_error)?;

    Ok(Json(
        json!({"message": "Password reset code sent to email"}),
    ))
}

/// ステップ2: リセットコード検証とトークン発行
async fn handle_verify_reset_code(
    State(state): State<AppState>,
    _headers: axum::http::HeaderMap,
    jar: CookieJar,
    Json(req): Json<VerifyResetCodeRequest>,
) -> Result<impl IntoResponse, Response> {
    let reset_token = state
        .auth_service
        .verify_reset_code_and_issue_reset_token(req.email, req.code)
        .await
        .map_err(map_error)?;

    // リセットトークンをCookieに設定（30分間有効）
    let cookie_config = state.config.server.get_cookie_config();

    let reset_cookie = Cookie::build(("reset_token", reset_token))
        .path("/")
        .max_age(time::Duration::seconds(30 * 60))
        .same_site(cookie_config.same_site)
        .secure(cookie_config.secure)
        .http_only(cookie_config.http_only)
        .build();

    Ok((
        jar.add(reset_cookie),
        Json(json!({"message": "Code verified successfully"})),
    ))
}

/// ステップ3: パスワードリセット完了
async fn handle_complete_password_reset(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<CompletePasswordResetRequest>,
) -> Result<impl IntoResponse, Response> {
    // Cookieからリセットトークンを取得
    let reset_token = jar
        .get("reset_token")
        .ok_or_else(|| {
            map_error(AppError::ValidationError(
                "Reset token not found".to_string(),
            ))
        })?
        .value()
        .to_string();

    state
        .auth_service
        .complete_password_reset(&reset_token, &req.email, &req.new_password)
        .await
        .map_err(map_error)?;

    // リセットトークンCookieを削除
    let jar = jar.remove(Cookie::from("reset_token"));

    Ok((jar, Json(json!({"message": "Password reset successful"}))))
}
