use axum::{Router, routing::post};

pub fn create_ai_routes() -> Router {
    Router::new().route("/chat", post(chat))
}

async fn chat() -> &'static str {
    "AI Chat endpoint"
}