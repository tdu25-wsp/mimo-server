use axum::{Router, routing::{get, post}};

pub fn create_memos_routes() -> Router {
    Router::new()
        .route("/", get(list_memos))
        .route("/", post(create_memo))
}

async fn list_memos() -> &'static str {
    "List memos"
}

async fn create_memo() -> &'static str {
    "Create memo"
}