use axum::{
    Router,
    http::{HeaderValue, Method},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::auth::create_decoding_key;
use crate::config::Config;
use crate::routes::{create_api_routes, create_share_routes};
use crate::services::{AuthService, MemoService, SummaryService, TagService};
use jsonwebtoken::DecodingKey;

/// アプリケーション全体で共有される状態
#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    /// JWT検証用のデコード鍵（一度だけ生成して再利用）
    pub jwt_decoding_key: DecodingKey,
    /// サービス層
    pub auth_service: Arc<AuthService>,
    pub memo_service: Arc<MemoService>,
    pub summary_service: Arc<SummaryService>,
    pub tag_service: Arc<TagService>,
    /// アプリケーション設定
    pub config: Arc<Config>,
}

pub async fn start_server(
    addr: SocketAddr,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting Mimo Server...");

    println!("Configuring CORS...");
    let allowed_origins = [
        "https://mimo.shuta.me".parse::<HeaderValue>().unwrap(),
        format!("http://localhost:{}", addr.port())
            .parse::<HeaderValue>()
            .unwrap(),
        format!("http://127.0.0.1:{}", addr.port())
            .parse::<HeaderValue>()
            .unwrap(),
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        format!("http://{}", addr).parse::<HeaderValue>().unwrap(),
        format!("https://{}", addr).parse::<HeaderValue>().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
        ]);

    println!("Creating routes...");
    let app = Router::new()
        .nest("/api", create_api_routes())
        .nest("/share", create_share_routes())
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind(addr).await?;
    println!("Server is running on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
