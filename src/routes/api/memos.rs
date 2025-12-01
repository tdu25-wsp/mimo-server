use axum::{
    Router,
    routing::{delete, get, patch, post},
    response::{IntoResponse, Json},
};
use axum_extra::extract::cookie::CookieJar;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct Memo {
    MemoID: String,
    UserID: String,
    TagID: String,
    autoTagID: String,

    shareUrlToken: Option<String>,
    createdAt: DateTime<Utc>,
    updatedAt: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct Memos {
    memos: Vec<Memo>,
}

pub fn create_memo_routes() -> Router {
    Router::new()
        .route(
            "/memos",
            post(handle_create_memo)
                .get(handle_get_memos)
                .delete(handle_delete_memos),
        )
        .route("/memos/:id", get(handle_get_memo).patch(handle_update_memo))
}

async fn handle_create_memo() -> impl IntoResponse {
}

fn handle_get_memos() -> impl IntoResponse {
    // Implementation here
    todo!()
}

fn handle_delete_memos() {
    // Implementation here
    todo!()
}

fn handle_get_memo() {
    // Implementation here
    todo!()
}
fn handle_update_memo() {
    // Implementation here
    todo!()
}
