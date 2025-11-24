//mod.rsに機能を統合しているため、ある意味はない
use axum::Router;

pub async fn create_api_routes() -> Router {
    Router::new()
        .nest("/api", create_auth_routes())
        .nest("/api", create_memos_routes())
        .nest("/api", create_settings_routes())
        .nest("/api", create_tags_routes())
        .nest("/api", create_ai_routes())
}
