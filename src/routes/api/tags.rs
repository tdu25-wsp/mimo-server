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

#[derive(Serialize, Deserialize)]
struct Tags {
    tags: Vec<Tag>,
}

pub fn create_tags_routes() -> Router<AppState> {
    Router::new().route("/tags", post(handle_create_tag)).route(
        "/tags/{capture}",
        get(handle_get_tag_list)
            .patch(handle_update_tag)
            .delete(handle_delete_tag),
    )
}

//// ハンドラ関数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tag {
    tag_id: String,
    user_id: String,
    name: String,
    color_code: String,

    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TagList {
    tags: Vec<Tag>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTagRequest {
    name: String,
    color_code: String,
}

async fn handle_create_tag(
    jar: CookieJar,
    req: extract::Json<CreateTagRequest>,
) -> impl IntoResponse {
    // TODO: Write tag to DB

    Json(json!({
        "message": "Tag created successfully",
        "tag": Tag {
            tag_id: "tag_123".to_string(),
            user_id: "user_456".to_string(),
            name: req.name.clone(),
            color_code: req.color_code.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now()
            }
        }
    ))
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
