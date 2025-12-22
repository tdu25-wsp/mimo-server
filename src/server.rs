use axum::{Router, http::Method};
use jsonwebtoken::DecodingKey;
use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::config::Config;
use crate::routes::{create_api_routes, create_share_routes};
use crate::services::{AuthService, MemoService, SummaryService, TagService};

/// アプリケーション全体で共有される状態
#[derive(Clone)]
pub struct AppState {
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
    println!("Environment: {:?}", state.config.server.env);

    let allowed_origins = state
        .config
        .server
        .get_allowed_origins(&addr)
        .map_err(|e| format!("Failed to configure CORS origins: {}", e))?;

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_credentials(true)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
        ])
        .max_age(Duration::from_secs(180));

    println!("Creating routes...");
    let app = Router::new()
        .nest("/api", create_api_routes())
        //.nest("/share", create_share_routes())
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind(addr).await?;
    println!("Server is running on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
