use std::net::SocketAddr;

mod routes;
mod api;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:5050"
        .parse()
        .map_err(|e| format!("Failed to parse SocketAddr: {}", e))?;

    server::start_server(addr).await?;
    Ok(())
}
