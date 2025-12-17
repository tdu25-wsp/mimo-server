use axum::{
    Json,
    Router,
    extract::Path,
    response::IntoResponse,
    routing::get
};
use axum_extra::extract::CookieJar;
use serde_json::json;

use crate::repositories::Memo;

pub fn create_share_routes() -> Router {
    Router::new().route("/{capture}", get(handle_get_share))
        .route("/test", get(handle_test))
}

async fn handle_get_share(jar: CookieJar, req: Path<String>) -> impl IntoResponse {
    Json(Memo {
        memo_id: "hoge".to_string(),
        user_id: "user_123".to_string(),
        content: "This is a shared memo.".to_string(),
        tag_id: "tag_456".to_string(),
        auto_tag_id: "auto_tag_123".to_string(),
        manual_tag_id: Some(vec!["manual_tag_456".to_string()]),
        share_url_token: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

async fn handle_test() -> impl IntoResponse {
    Json(json!({"status": "share service is running"}))


}
