use axum::{
    Router,
    http::{HeaderValue, Method},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer; // Axum 0.8で必要

// エラーになっていた関数インポートを追加
use crate::routes::{create_api_routes, create_share_routes};

pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Mimo Server...");

    println!("Configuring CORS...");
    let cors = CorsLayer::new()
        .allow_origin("https://mimo.shuta.me".parse::<HeaderValue>().unwrap())
        .allow_origin(format!("http://localhost{}", addr.port()).parse::<HeaderValue>().unwrap())
        .allow_origin(format!("http://{}", addr).parse::<HeaderValue>().unwrap())
        .allow_origin(format!("https://{}", addr).parse::<HeaderValue>().unwrap())
        // 文字列の "GET" だとエラーになるため、Method::GET に修正
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
        ]);

    print!("Creating routes...\n");
    let app = Router::new()
        .nest("/api", create_api_routes())
        .nest("/share", create_share_routes())
        .layer(cors);

    let listener = TcpListener::bind(addr).await?;
    print!("Server is running on http://{}\n", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
