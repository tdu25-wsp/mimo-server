use axum::{
    Router,
    routing::{get, post},
    response::{IntoResponse, Json},
};

pub fn create_ai_routes() -> Router {
    Router::new()
        .route("/ai/summarize", post(summarize_memo))
        .route("/ai/journaling", post(create_journal))
}

async fn summarize_memo() -> impl IntoResponse {
    // Implementation here
    todo!()
}

async fn create_journal() -> impl IntoResponse {
    // Implementation here
    todo!()
}
