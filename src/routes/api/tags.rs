use axum::{
    Router, extract, routing::{delete, get, patch, post},
    response::{IntoResponse, Json},
};

use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use serfe_json::{json, Value};
use chrono::{DateTime, Utc};


#[derive(Serialize, Deserialize)]
struct Tags {
    tags: Vec<Tag>,
}

pub fn create_tags_routes() -> Router {
    Router::new()
        .route("/tags", post(handle_create_tag).get(handle_get_tag_list))
        .route(
            "/tags/:id",
            patch(handle_update_tag).delete(handle_delete_tag),
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTagRequest {
    name: String,
    color_code: String,
}

async fn handle_create_tag(jar: CookieJar, req: extract::Json<CreateTagRequest>) -> impl IntoResponse {
    
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

async fn handle_get_tag_list() -> impl IntoResponse {
    // Implementation here
    todo!()
}

async fn handle_update_tag() -> impl IntoResponse {
    // Implementation here
    todo!()
}

async fn handle_delete_tag() -> impl IntoResponse {
    // Implementation here
    todo!()
}
