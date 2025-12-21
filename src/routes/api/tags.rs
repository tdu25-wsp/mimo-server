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

use crate::auth::extract_user_id_from_token;
use crate::server::AppState;
use crate::repositories::{Tag, TagList, CreateTagRequest, UpdateTagRequest};
use crate::error::{AppError, Result};

pub fn create_tags_routes() -> Router<AppState> {
    Router::new()
        .route("/tags/{capture}",get(handle_get_tag_list))
        .route("/tags/{capture}", post(handle_create_tag))
        .route("/tags/{capture}", patch(handle_update_tag))
        .route("/tags/{capture}", delete(handle_delete_tag))
}

async fn handle_create_tag(
    State(state): State<AppState>, 
    jar: CookieJar,
    Path(user_id): Path<String>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>> {
    let access_token = jar
        .get("access_token")
        .ok_or(AppError::Unauthorized("Authentication required".to_string()))?;

    let authenticated_user_id = extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let tag = state.tag_service.create_tag(&user_id, req).await?;
    Ok(Json(tag))
}

async fn handle_get_tag_list(
    jar: CookieJar,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<TagList>> {
    let access_token = jar
        .get("access_token")
        .ok_or(AppError::Unauthorized("Authentication required".to_string()))?;

    let authenticated_user_id = extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let tags = state.tag_service.get_tags_by_user(&user_id).await?;
    Ok(Json(TagList { tags }))
}

async fn handle_update_tag(
    jar: CookieJar,
    Path(tag_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<Tag>> {
    let access_token = jar
        .get("access_token")
        .ok_or(AppError::Unauthorized("Authentication required".to_string()))?;

    let authenticated_user_id = extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    let updated_tag = state.tag_service.update_tag(&authenticated_user_id, &tag_id, req).await?;
    Ok(Json(updated_tag))
}

async fn handle_delete_tag(
    jar: CookieJar,
    Path(tag_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>> {
    let access_token = jar
        .get("access_token")
        .ok_or(AppError::Unauthorized("Authentication required".to_string()))?;

    let authenticated_user_id = extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key)?;

    state.tag_service.delete_tag(&authenticated_user_id, &tag_id).await?;
    Ok(Json(json!({"status": format!("tag_id: {} deleted", tag_id)})))
}
