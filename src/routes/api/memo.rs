use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, patch, post},
};
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;

use crate::{
    auth::extract_user_id_from_token,
    error::{AppError, Result},
    repositories::{Memo, MemoCreateRequest, MemoList, MemoUpdateRequest},
    server::AppState,
};

pub fn create_memo_routes() -> Router<AppState> {
    Router::new()
        .route("/memos/list/{capture}", get(list_memos))
        .route("/memos", post(create_memo))
        .route("/memos/{capture}", patch(update_memo))
        .route("/memos/{capture}", get(get_memo))
        .route("/memos/{capture}", delete(delete_memo))
}

async fn list_memos(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id): Path<String>,
) -> Result<Json<MemoList>> {
    let access_token = jar.get("access_token").ok_or(AppError::Unauthorized(
        "Authentication required".to_string(),
    ))?;

    let authenticated_user_id =
        extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let memos = state.memo_service.find_by_user(&user_id).await?;
    Ok(Json(MemoList { memos }))
}

async fn create_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<MemoCreateRequest>,
) -> Result<Json<Memo>> {
    let access_token = jar.get("access_token").ok_or(AppError::Unauthorized(
        "Authentication required".to_string(),
    ))?;

    let authenticated_user_id =
        extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    // リクエストのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != req.user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let memo = state.memo_service.create(req).await?;
    Ok(Json(memo))
}

async fn get_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<Memo>> {
    let access_token = jar.get("access_token").ok_or(AppError::Unauthorized(
        "Authentication required".to_string(),
    ))?;

    let authenticated_user_id =
        extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    let memo = state.memo_service.find_by_id(&id).await?;

    // メモの所有者と認証されたユーザーIDが一致するか確認
    if memo.user_id != authenticated_user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    Ok(Json(memo))
}

async fn update_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Json(req): Json<MemoUpdateRequest>,
) -> Result<Json<Memo>> {
    let access_token = jar.get("access_token").ok_or(AppError::Unauthorized(
        "Authentication required".to_string(),
    ))?;

    let authenticated_user_id =
        extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    // 更新前にメモの所有者を確認
    let existing_memo = state.memo_service.find_by_id(&id).await?;
    if existing_memo.user_id != authenticated_user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let memo = state.memo_service.update_content(&id, req.content).await?;
    Ok(Json(memo))
}

async fn delete_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let access_token = jar.get("access_token").ok_or(AppError::Unauthorized(
        "Authentication required".to_string(),
    ))?;

    let authenticated_user_id =
        extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    // 削除前にメモの所有者を確認
    let existing_memo = state.memo_service.find_by_id(&id).await?;
    if existing_memo.user_id != authenticated_user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    state.memo_service.delete(&id).await?;
    Ok(Json(json!({
        "status": "success",
        "message": format!("Memo deletion completed: {id}")
    })))
}
