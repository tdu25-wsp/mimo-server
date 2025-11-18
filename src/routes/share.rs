use axum::{Router, routing::get};

async fn create_share_routes() -> Router {
    Router::new().route("/share/:id", get(handle_get_share))
}
