use axum::{
    Router,
    routing::{delete, get, patch, post},
};

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

fn handle_create_memo() {
    // Implementation here
    todo!()
}

fn handle_get_memos() {
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
