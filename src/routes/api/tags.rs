use axum::{Router, routing::get};

pub fn create_tags_routes() -> Router {
    Router::new().route("/", get(list_tags))
}

async fn list_tags() -> &'static str {
    "List tags"
}