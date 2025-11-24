use axum::{
    Router,
    routing::{delete, get, patch, post},
};

pub fn create_tags_routes() -> Router {
    Router::new()
        .route("/tags", post(handle_create_tag).get(handle_get_tag_list))
        .route("/tags/:id", patach(handle_update_tag).delete(handle_delete_tag))
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