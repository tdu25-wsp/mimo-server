use axum::{Router, routing::get};

use crate::server::AppState;

pub fn create_settings_routes() -> Router<AppState> {
    Router::new().route(
        "/settings",
        get(handle_get_settings).patch(handle_update_settings),
    )
}

async fn handle_get_settings() -> &'static str {
    "Get Settings - To be implemented"
}

async fn handle_update_settings() -> &'static str {
    "Update Settings - To be implemented"
}
