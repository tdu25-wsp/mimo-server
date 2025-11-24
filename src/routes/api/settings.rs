use axum::{Router, routing::get};

pub fn create_settings_routes() -> Router {
    Router::new().route("/", get(get_settings))
}

async fn get_settings() -> &'static str {
    "Get settings"
}