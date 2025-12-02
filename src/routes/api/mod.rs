use axum::Router;

mod ai;
mod auth;
mod memo;
mod settings;
mod tags;

use ai::create_ai_routes;
use auth::create_auth_routes;
use memo::create_memo_routes;
use settings::create_settings_routes;
use tags::create_tags_routes;

pub fn create_api_routes() -> Router {
    Router::new()
        .nest("", create_auth_routes)
        .nest("", create_ai_routes)
        .nest("", create_memo_routes)
        .nest("", create_tags_routes)
        .nest("", create_settings_routes)
}

