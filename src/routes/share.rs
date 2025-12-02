use axum::{
    extract::Path,
    routing::get,
    Router,
};

pub async fn create_share_routes() -> Router {
    Router::new().route("/{id}", get(handle_get_share))
}

async fn handle_get_share(Path(id): Path<String>) -> String {
    format!("Shared content for ID: {}", id)
}