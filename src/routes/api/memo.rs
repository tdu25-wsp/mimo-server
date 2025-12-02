use axum::{
    Router,
    routing::{delete, get, patch, post},
    response::{IntoResponse, Json},
    extract,
};
use axum_extra::extract::cookie::CookieJar;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::{de, json};


pub fn create_memo_routes() -> Router {
    Router::new()
        .route(
            "/memos",
            post(handle_create_memo)
                .get(handle_get_memos)
                .delete(handle_delete_memos),
        )
        .route("/memos/:id", get(handle_get_memo).patch(handle_update_memo))
}

//// データ構造定義
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Memo {
    memo_id: String,
    content: String,

    user_id: String,
    tag_id: String,
    auto_tag_id: String,
    manual_tag_id: Option<String>,

    share_url_token: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct MemoCreateRequest {
    content: String,
    tag_id: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct MemoList {
    memos: Vec<Memo>,
}

#[derive(Deserialize)]
struct MemoRequest {
    memo_id: Vec<String>,
}
    

//// ハンドラ関数
async fn handle_create_memo(jar: CookieJar, req: extract::Json<MemoCreateRequest>) -> impl IntoResponse {
    let access_token = jar.get("access_token");
    // TODO: Validate access token

    //TODO: Write DB

    Json(json!({
        "status": "success",
        "message": "Memo created successfully"
    }))
}

async fn handle_get_memos(jar: CookieJar, req: extract::Json<MemoRequest>) -> impl IntoResponse {
    let access_token = jar.get("access_token");
    // TODO: Validate access token

    Json(MemoList {
        memos: vec![], // TODO: Fetch memos from DB
    })
}

async fn handle_delete_memos(jar: CookieJar, req: extract::Json<MemoRequest>) -> impl IntoResponse {
    let access_token = jar.get("access_token");
    //TODO: Validate access token

    //TODO: Delete memos from DB
    let memo_ids = &req.memo_id;

    Json(json!({
        "status": "success",
        "message": "Memos deleted successfully",
        "deleted_memo_ids": memo_ids,
    }))
}

async fn handle_get_memo(jar: CookieJar, req: extract::Path<String>) -> impl IntoResponse {
    let access_token = jar.get("access_token");

    // TODO: Fetch memo from DB
    Json(Memo {
        memo_id: String::new(),
        content: String::new(),
        user_id: String::new(),
        tag_id: String::new(),
        auto_tag_id: String::new(),
        manual_tag_id: None,
        share_url_token: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
}
async fn handle_update_memo(jar: CookieJar, memo_id: extract::Path<String>, req: extract::Json<Memo>) -> impl IntoResponse {
    let access_token = jar.get("access_token");

    // TODO: Update memo in DB

    Json(json!({
        "status": "success",
        "message": format!("Memo {} updated successfully", memo_id.to_string()),
        })
    )
}
