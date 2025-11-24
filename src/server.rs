use axum::{Router, http::{HeaderValue, Method}};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tokio::net::TcpListener; // Axum 0.8で必要

// エラーになっていた関数インポートを追加
use crate::routes::{create_api_routes, create_share_routes};

pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Mimo Server...");

    let cors = CorsLayer::new()
        .allow_origin("https://mimo.shuta.me".parse::<HeaderValue>().unwrap())
        .allow_origin(format!("http://{}", addr).parse::<HeaderValue>().unwrap())
        .allow_origin(format!("https://{}", addr).parse::<HeaderValue>().unwrap())
        // 文字列の "GET" だとエラーになるため、Method::GET に修正
        .allow_methods(vec![Method::GET, Method::POST, Method::PATCH, Method::DELETE]);

    let app = Router::new()
        .nest("/api", create_api_routes().await)
        .nest("/share", create_share_routes().await)
        .layer(cors);

    // 【修正】Axum 0.8の書き方に変更 
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}