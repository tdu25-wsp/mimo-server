use axum::{Router, extract::Path, routing::get};

pub fn create_share_routes() -> Router {
    Router::new().route("/share/:id", get(handle_get_share))
}

fn handle_get_share() {
    // Implementation here
    todo!()
}
