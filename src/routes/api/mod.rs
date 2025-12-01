use axum::Router;

mod ai;
mod auth;
mod memos;
mod settings;
mod tags;

pub fn create_api_routes() -> Router {
    Router.new()
        .nest("", create_auth_routes)
        .nest("", create_ai_routes)
        .nest("", create_memos_routes)
        .nest("", create_tags_routes)
        .nest("", create_settings_routes)
}

