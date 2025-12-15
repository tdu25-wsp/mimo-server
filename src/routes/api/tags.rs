use axum::{
    Router, extract,
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post},
};

use axum_extra::extract::CookieJar;
use serde_json::json;
use std::sync::Arc;
use crate::{
    services::TagService,
    repositories::{TagList, TagCreateRequest, TagUpdateRequest},
    error::Result,
};

pub fn create_tags_routes(service: Arc<TagService>) -> Router {
    Router::new()
        .route("/tags", {
            let service = service.clone();
            post(move |jar, json| handle_create_tag(jar, service.clone(), json))
        })
        .route("/tags", {
            let service = service.clone();
            get(move |jar| handle_get_tag_list(jar, service.clone()))
        })
        .route("/tags/{capture}", {
            let service = service.clone();
            patch(move |jar, path, json| handle_update_tag(jar, service.clone(), path, json))
        })
        .route("/tags/{capture}", {
            // let service = service.clone(); // deleteで使用
            delete(move |jar, path| handle_delete_tag(jar, service.clone(), path))
        })
}

async fn handle_create_tag(
    jar: CookieJar,
    service: Arc<TagService>,
    extract::Json(req): extract::Json<TagCreateRequest>,
) -> Result<impl IntoResponse> {
    let _access_token = jar.get("access_token");
    // TODO: Validate user from token
    let user_id = "user_123".to_string(); // Dummy user

    let tag = service.create_tag(user_id, req).await?;
    
    Ok(Json(json!({
        "message": "Tag created successfully",
        "tag": tag
    })))
}

async fn handle_get_tag_list(
    jar: CookieJar,
    service: Arc<TagService>,
) -> Result<impl IntoResponse> {
    let _access_token = jar.get("access_token");
    let user_id = "user_123".to_string(); // Dummy user

    let tags = service.get_user_tags(&user_id).await?;
    Ok(Json(TagList { tags }))
}

async fn handle_update_tag(
    jar: CookieJar,
    service: Arc<TagService>,
    extract::Path(tag_id): extract::Path<String>,
    extract::Json(req): extract::Json<TagUpdateRequest>,
) -> Result<impl IntoResponse> {
    let _access_token = jar.get("access_token");
    
    let tag = service.update_tag(&tag_id, req).await?;
    Ok(Json(json!({
        "message": format!("Tag {} updated successfully",tag.tag_id),
        "tag": tag
    })))
}

async fn handle_delete_tag(
    jar: CookieJar,
    service: Arc<TagService>,
    extract::Path(tag_id): extract::Path<String>,
) -> Result<impl IntoResponse> {
    let _access_token = jar.get("access_token");
    
    service.delete_tag(&tag_id).await?;
    Ok(Json(json!({
        "message": format!("Tag {} deleted successfully", tag_id),
    })))
}