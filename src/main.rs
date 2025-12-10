use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Context;
use mongodb::Client as MongoClient;
use sqlx::postgres::PgPoolOptions;

mod config;
mod error;
mod repositories;
mod services;
mod routes;
mod server;

use config::Config;
use repositories::MongoMemoRepository;
use services::MemoService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Loading configuration...");
    let config = Config::load()?;

    println!("Connecting to databases...");

    // PostgreSQL 接続（認証用）
    let postgres_url = format!("{}/{}", 
        config.database.postgres.connection_url, 
        config.database.postgres.db_name
    );
    let _pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    // MongoDB 接続（メモ用）
    let mongo_client = MongoClient::with_uri_str(&config.database.mongodb.connection_uri)
        .await
        .context("Failed to connect to MongoDB")?;
    let mongo_db = mongo_client.database(&config.database.mongodb.db_name);

    // サービスの構築
    let memo_service = Arc::new(MemoService::new(
        Arc::new(MongoMemoRepository::new(mongo_db)),
    ));

    // サーバー起動
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("Failed to parse SocketAddr")?;

    server::start_server(addr, memo_service)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    Ok(())
}
