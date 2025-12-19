use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, patch, post},
};
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;

use crate::{
    error::Result,
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
    // TODO: Validate access token
    let _access_token = jar.get("access_token");

    let memos = state.memo_service.find_by_user(&user_id).await?;
    Ok(Json(MemoList { memos }))
}

async fn create_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<MemoCreateRequest>,
) -> Result<Json<Memo>> {
    // TODO: Validate access token
    let _access_token = jar.get("access_token");

    let memo = state.memo_service.create(req).await?;
    Ok(Json(memo))
}

async fn get_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<Memo>> {
    // TODO: Validate access token
    let _access_token = jar.get("access_token");

    let memo = state.memo_service.find_by_id(&id).await?;
    Ok(Json(memo))
}

// 追加: メモ更新ハンドラー
async fn update_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Json(req): Json<MemoUpdateRequest>,
) -> Result<Json<Memo>> {
    let _access_token = jar.get("access_token");
    let memo = state.memo_service.update_content(&id, req.content).await?;
    Ok(Json(memo))
}

async fn delete_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Validate access token
    let _access_token = jar.get("access_token");

    state.memo_service.delete(&id).await?;
    Ok(Json(json!({
        "status": "success",
        "message": format!("Memo deletion completed: {id}")
    })))
}
