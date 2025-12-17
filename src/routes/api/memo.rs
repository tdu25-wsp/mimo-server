use axum::{
    Router,
    routing::{delete, get, post, patch}, // Added patch
    response::Json,
    extract::Path,
};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;
use serde_json::json;

use crate::{
    error::Result,
    repositories::{Memo, MemoCreateRequest, MemoList},
    services::MemoService,
};

pub fn create_memo_routes(service: Arc<MemoService>) -> Router {
    Router::new()
        .route("/users/{capture}/memos", {
            let service = service.clone();
            get(move |jar, path| list_memos(jar, service.clone(), path))
        })
        .route("/memos", {
            let service = service.clone();
            post(move |jar, json| create_memo(jar, service.clone(), json))
        })
        .route("/memos/{capture}", {
            let service = service.clone();
            get(move |jar, path| get_memo(jar, service.clone(), path))
        })
        .route("/memos/{capture}", {
            delete(move |jar, path| delete_memo(jar, service.clone(), path))
        })
}

async fn list_memos(
    jar: CookieJar,
    service: Arc<MemoService>,
    Path(user_id): Path<String>,
) -> Result<Json<MemoList>> {
    // TODO: Validate access token
    let _access_token = jar.get("access_token");
    
    let memos = service.find_by_user(&user_id).await?;
    Ok(Json(MemoList { memos }))
}

async fn create_memo(
    jar: CookieJar,
    service: Arc<MemoService>,
    Json(req): Json<MemoCreateRequest>,
) -> Result<Json<Memo>> {
    // TODO: Validate access token
    let _access_token = jar.get("access_token");
    
    let memo = service.create(req).await?;
    Ok(Json(memo))
}

async fn get_memo(
    jar: CookieJar,
    service: Arc<MemoService>,
    Path(id): Path<String>,
) -> Result<Json<Memo>> {
    // TODO: Validate access token
    let _access_token = jar.get("access_token");
    
    let memo = service.find_by_id(&id).await?;
    Ok(Json(memo))
}

// 追加: メモ更新ハンドラー
async fn update_memo(
    jar: CookieJar,
    service: Arc<MemoService>,
    Path(id): Path<String>,
    Json(req): Json<UpdateMemoRequest>,
) -> Result<Json<Memo>> {
    let _access_token = jar.get("access_token");
    let memo = service.update_content(&id, req.content).await?;
    Ok(Json(memo))
}

async fn delete_memo(
    jar: CookieJar,
    service: Arc<MemoService>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Validate access token
    let _access_token = jar.get("access_token");
    
    service.delete(&id).await?;
    Ok(Json(json!({
        "status": "success",
        "message": format!("Memo deletion completed: {id}")
    })))
}
