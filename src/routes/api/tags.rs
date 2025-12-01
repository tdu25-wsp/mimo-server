use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct Tag {
    TagID: String,
    UserID: String,
    Name: String,
    ColorCode: String,

    createdAt: DateTime<Utc>,
    updatedAt: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct Tags {
    tags: Vec<Tag>,
}

pub fn create_tags_routes() -> Router {
    Router::new()
        .route("/tags", post(handle_create_tag).get(handle_get_tag_list))
        .route(
            "/tags/:id",
            patach(handle_update_tag).delete(handle_delete_tag),
        )
}

fn handle_create_tag() {
    // Implementation here
    todo!()
}

fn handle_get_tag_list() {
    // Implementation here
    todo!()
}

fn handle_update_tag() {
    // Implementation here
    todo!()
}

fn handle_delete_tag() {
    // Implementation here
    todo!()
}
