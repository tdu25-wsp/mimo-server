use axum::{
    Router,
    extract::{Path, State},
    response::{Json, Response},
    routing::{delete, get, patch, post},
};

use axum_extra::extract::CookieJar;
use serde_json::{Value, json};

use crate::server::AppState;
use crate::repositories::{Tag, TagList, CreateTagRequest, UpdateTagRequest};
use crate::error::{AppError, map_error};

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
) -> std::result::Result<Json<Tag>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    let tag = state.tag_service.create_tag(&user_id, req).await.map_err(map_error)?;
    Ok(Json(tag))
}

async fn handle_get_tag_list(
    jar: CookieJar,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<Json<TagList>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    let tags = state.tag_service.get_tags_by_user(&user_id).await.map_err(map_error)?;
    Ok(Json(TagList { tags }))
}

async fn handle_update_tag(
    jar: CookieJar,
    Path(tag_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<UpdateTagRequest>,
) -> std::result::Result<Json<Tag>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    let updated_tag = state.tag_service.update_tag(&authenticated_user_id, &tag_id, req).await.map_err(map_error)?;
    Ok(Json(updated_tag))
}

async fn handle_delete_tag(
    jar: CookieJar,
    Path(tag_id): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<Json<Value>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    state.tag_service.delete_tag(&authenticated_user_id, &tag_id).await.map_err(map_error)?;
    Ok(Json(json!({"status": format!("tag_id: {} deleted", tag_id)})))
}
