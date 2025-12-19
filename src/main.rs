use anyhow::Context;
use mongodb::Client as MongoClient;
use sqlx::{Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};
use std::net::SocketAddr;
use std::sync::Arc;

mod auth;
mod config;
mod error;
mod repositories;
mod routes;
mod server;
mod services;

use auth::load_or_generate_secret_key;
use config::Config;
use repositories::{MemoRepository, SummaryRepository};
use server::AppState;
use services::{MemoService, SummaryService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Loading configuration...");
    let config = Config::load()?;

    println!("Connecting to databases...");

    // PostgreSQL 接続
    let postgres_url = format!(
        "{}/{}",
        config.database.postgres.connection_url, config.database.postgres.db_name
    );
    if !Postgres::database_exists(&postgres_url).await? {
        Postgres::create_database(&&postgres_url).await?;
        println!(
            "Created PostgreSQL database: {}",
            config.database.postgres.db_name
        );
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

    // JWT秘密鍵の読み込み（環境変数、ファイル、または自動生成）
    println!("Loading JWT secret key...");
    let jwt_secret = load_or_generate_secret_key(None)?;

    // サービスの構築
    let memo_service = Arc::new(MemoService::new(Arc::new(MemoRepository::new(
        mongo_db.clone(),
    ))));
    let summary_service = Arc::new(SummaryService::new(
        Arc::new(SummaryRepository::new(mongo_db.clone())),
        Arc::new(MemoRepository::new(mongo_db.clone())),
    ));

    // AppState の構築
    let state = AppState {
        pg_pool,
        mongo_db,
        jwt_secret,
        memo_service,
        summary_service,
        config: Arc::new(config.clone()),
    };

    // サーバー起動
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("Failed to parse SocketAddr")?;

    server::start_server(addr, state)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
