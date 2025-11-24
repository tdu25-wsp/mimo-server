use axum::{Router, routing::post};

pub fn create_auth_routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

async fn login() -> &'static str {
    "Login endpoint"
}

async fn register() -> &'static str {
    "Register endpoint"
}