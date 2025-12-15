use axum::{
    Router,
    http::{HeaderValue, Method},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer; 

use crate::routes::{create_api_routes, create_share_routes};
use crate::services::{MemoService, SummaryService}; // memo_serviceに加えてsummary_serviceもインポート

pub async fn start_server(
    addr: SocketAddr, 
    memo_service: Arc<MemoService>,
    summary_service: Arc<SummaryService>, // summary_serviceを引数に追加
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting Mimo Server...");

    println!("Configuring CORS...");
    let cors = CorsLayer::new()
        .allow_origin("https://mimo.shuta.me".parse::<HeaderValue>().unwrap())
        .allow_origin(
            format!("http://localhost{}", addr.port())
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_origin(format!("http://{}", addr).parse::<HeaderValue>().unwrap())
        .allow_origin(format!("https://{}", addr).parse::<HeaderValue>().unwrap())
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
        ]);

    println!("Creating routes...");
    let app = Router::new()
        .nest("/api", create_api_routes(memo_service, summary_service))
        .nest("/share", create_share_routes())
        .layer(cors);

    let listener = TcpListener::bind(addr).await?;
    println!("Server is running on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
