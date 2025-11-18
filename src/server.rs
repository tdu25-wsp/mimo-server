use axum::{Router, Server};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use crate::routes::share::create_share_routes;

pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Mimo Server...");

    let cors = CorsLayer::new()
        .allow_origin("https://mimo.shuta.me".parse::<HeaderValue>().unwrap())
        .allow_origin(format!("http://{}", addr).parse::<HeaderValue>().unwrap())
        .allow_origin(format!("https://{}", addr).parse::<HeaderValue>().unwrap())
        .allow_methods(vec!["GET", "POST", "PATCH", "DELETE"]);

    let app = Router::new()
        .nest("/api", create_api_routes().await)
        .nest("/share", create_share_routes().await);

    Server::bind(&addr).serve(app.layer(cors).into_make_service());

    Ok(())
}
