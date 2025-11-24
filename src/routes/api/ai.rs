use axum::{
    Router,
    routing::{get, post},
};

pub fn create_ai_routes() -> Router {
    Router::new()
        .route("/ai/summarize", post(summarize_memo))
        .route("/ai/journaling", post(create_journal))
}

fn summarize_memo() {
    // Implementation here
    todo!()
}

fn create_journal() {
    // Implementation here
    todo!()
}
