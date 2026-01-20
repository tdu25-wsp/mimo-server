use axum::{
    Router,
    extract::{Path, State},
    response::{Json, Response},
    routing::{delete, get, patch, post},
};
use axum_extra::extract::CookieJar;
use serde_json::json;

use crate::{
    error::{AppError, map_error},
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
) -> std::result::Result<Json<MemoList>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    let memos = state.memo_service.find_by_user(&user_id).await.map_err(map_error)?;
    Ok(Json(MemoList { memos }))
}

async fn create_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<MemoCreateRequest>,
) -> std::result::Result<Json<Memo>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    // リクエストのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != req.user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    let memo = state.memo_service.create(req).await.map_err(map_error)?;
    Ok(Json(memo))
}

async fn get_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> std::result::Result<Json<Memo>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    let memo = state.memo_service.find_by_id(&id).await.map_err(map_error)?;

    // メモの所有者と認証されたユーザーIDが一致するか確認
    if memo.user_id != authenticated_user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    Ok(Json(memo))
}

async fn update_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Json(req): Json<MemoUpdateRequest>,
) -> std::result::Result<Json<Memo>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    // 更新前にメモの所有者を確認
    let existing_memo = state.memo_service.find_by_id(&id).await.map_err(map_error)?;
    if existing_memo.user_id != authenticated_user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    let memo = state.memo_service.update_memo(&id, req).await.map_err(map_error)?;
    Ok(Json(memo))
}

async fn delete_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    // 削除前にメモの所有者を確認
    let existing_memo = state.memo_service.find_by_id(&id).await.map_err(map_error)?;
    if existing_memo.user_id != authenticated_user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    state.memo_service.delete(&id).await.map_err(map_error)?;
    Ok(Json(json!({
        "status": "success",
        "message": format!("Memo deletion completed: {id}")
    })))
}
