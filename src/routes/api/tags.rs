use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post},
};

use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use mongodb::action::Update;
use serde::{Deserialize, Serialize, de};
use serde_json::{Value, json};

use crate::server::AppState;
use crate::repositories::{Tag, TagList};
use crate::error::Result;
use crate::repositories::{Tag, TagList, CreateTagRequest, UpdateTagRequest};

pub fn create_tags_routes() -> Router<AppState> {
    Router::new()
        .route("/tags/{capture}",get(handle_get_tag_list))
        .route("/tags/{capture}", post(handle_create_tag))
        .route("/tags/{capture}", patch(handle_update_tag))
        .route("/tags/{capture}", delete(handle_delete_tag))
}

//// ハンドラ関数
async fn handle_create_tag(
    State(state): State<AppState>, 
    jar: CookieJar,
    Path(user_id): Path<String>,
    state: State<AppState>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>> {
    //TODO validate token
    let tag = state.tag_service.create_tag(&user_id, req).await?;
    Ok(Json(tag))
}

async fn handle_get_tag_list(
    jar: CookieJar,
    Path(user_id): Path<String>,
    state: State<AppState>,
) -> Result<Json<TagList>> {
    // TODO: validate access token
    let tags = state.tag_service.get_tags_by_user(&user_id).await?;
    Ok(Json(TagList { tags }))
}

async fn handle_update_tag(
    jar: CookieJar,
    Path(tag_id): Path<String>,
    state: State<AppState>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<Tag>> {
    // TODO: validate access token
    let user_id = ""; // Extract user_id from token in real implementation
    let updated_tag = state.tag_service.update_tag(&user_id, &tag_id, req).await?;
    Ok(Json(updated_tag))
}

async fn handle_delete_tag(
    jar: CookieJar,
    Path(tag_id): Path<String>,
    state: State<AppState>,
) -> Result<Json<Value>> {
    // TODO: validate access token
    let user_id = ""; // Extract user_id from token in real implementation
    state.tag_service.delete_tag(&user_id, &tag_id).await?;
    Ok(Json(json!({"status": format!("tag_id: {} deleted", tag_id)})))
}