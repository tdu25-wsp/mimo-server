use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Context;
use mongodb::Client as MongoClient;
use sqlx::{postgres::PgPoolOptions, Postgres, migrate::MigrateDatabase};

mod config;
mod error;
mod repositories;
mod services;
mod routes;
mod server;

use config::Config;
use repositories::{MemoRepository, SummaryRepository};
use services::{MemoService, SummaryService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Loading configuration...");
    let config = Config::load()?;

    println!("Connecting to databases...");

    // PostgreSQL 接続
    let postgres_url = format!("{}/{}", 
        config.database.postgres.connection_url, 
        config.database.postgres.db_name
    );
    if !Postgres::database_exists(&postgres_url).await? {
        Postgres::create_database(&&postgres_url).await?;
        println!("Created PostgreSQL database: {}", config.database.postgres.db_name);
    }
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await
        .context("Failed to connect to PostgreSQL")?;
    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .context("Failed to run PostgreSQL migrations")?;

    // MongoDB 接続（メモ用）
    let mongo_client = MongoClient::with_uri_str(&config.database.mongodb.connection_uri)
        .await
        .context("Failed to connect to MongoDB")?;
    let mongo_db = mongo_client.database(&config.database.mongodb.db_name);

    // サービスの構築
    let memo_service = Arc::new(MemoService::new(
        Arc::new(MemoRepository::new(mongo_db.clone())),
    ));
    let summary_service = Arc::new(SummaryService::new(
        Arc::new(SummaryRepository::new(mongo_db.clone())),
          Arc::new(MemoRepository::new(mongo_db))
      ));

    // サーバー起動
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("Failed to parse SocketAddr")?;

    server::start_server(addr, memo_service, summary_service)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    Ok(())
}
