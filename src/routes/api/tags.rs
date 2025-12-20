use axum::{
    Router,
    extract::{self, State},
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post},
};

use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de};
use serde_json::{Value, json};

use crate::server::AppState;
use crate::repositories::{Tag, TagList};

#[derive(Serialize, Deserialize)]
struct Tags {
    tags: Vec<Tag>,
}

pub fn create_tags_routes() -> Router<AppState> {
    Router::new().route("/tags", post(handle_create_tag))
    .route("/tags/recommend", post(handle_recommend_tag))
    .route(
        "/tags/{capture}",
        get(handle_get_tag_list)
            .patch(handle_update_tag)
            .delete(handle_delete_tag),
    )
}

//// ハンドラ関数

#[derive(Deserialize)]
struct RecommendTagRequest {
    content: String,
}
// タグ推薦ハンドラ
async fn handle_recommend_tag(
    State(state): State<AppState>, //
    jar: CookieJar,
    extract::Json(req): extract::Json<RecommendTagRequest>, // リクエストボディの抽出
) -> impl IntoResponse {
    // ユーザーIDはいったん固定します(TODO)
    let user_id = "user123";

    match state.tag_service.recommend_tag(user_id, &req.content).await { // サービス層の推薦メソッドを呼び出し
        Ok(Some(tag_id)) => Json(json!({
            "status": "success", // ステータス
            "recommended_tag_id": tag_id // 推薦されたタグID
        })).into_response(), // 成功レスポンス
        Ok(None) => Json(json!({
            "status": "success",
            "message": "No suitable tag found",
            "recommended_tag_id": null
        })).into_response(), // タグが見つからなかった場合
        Err(e) => e.into_response(),
    }
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTagRequest {
    name: String,
    color_code: String,
}

async fn handle_create_tag(
    State(state): State<AppState>, 
    jar: CookieJar,
    extract::Json(req): extract::Json<CreateTagRequest>,
) -> impl IntoResponse {
    let _access_token = jar.get("access_token");
    // 一旦ユーザーIDは固定します(TODO)
    let user_id = "user123".to_string();

    // Serviceを呼び出してDBに保存
    match state.tag_service.create_tag(user_id, req.name, req.color_code).await {
        Ok(tag) => Json(json!({
            "message": "Tag created successfully",
            "tag": tag
        })).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn handle_get_tag_list(jar: CookieJar) -> impl IntoResponse {
    //TODO: Fetch tags from DB
    Json(TagList {
        tags: vec![], // TODO: Fetch tags from DB
    })
}

async fn handle_update_tag(jar: CookieJar, req: extract::Path<String>) -> impl IntoResponse {
    //TODO: Update tag in DB
    Json(json!({
        "message": format!("Tag {} updated successfully", req.to_string()),
    }))
}

async fn handle_delete_tag(jar: CookieJar, req: extract::Path<String>) -> impl IntoResponse {
    //TODO: Delete tag from DB
    Json(json!({
        "message": format!("Tag {} deleted successfully", req.to_string()),
    }))
}
