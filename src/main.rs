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

use config::Config;
use repositories::{MemoRepository, SummaryRepository, TagRepository};
use server::AppState;
use services::{AuthService, MemoService, SummaryService, TagService};

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
    } else {
        println!(
            "PostgreSQL database {} already exists",
            config.database.postgres.db_name
        );
    }

    println!("Connecting to PostgreSQL at {}", &postgres_url);
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
    println!(
        "Connecting to MongoDB at {}",
        &config.database.mongodb.connection_uri
    );
    let mongo_client = MongoClient::with_uri_str(&config.database.mongodb.connection_uri)
        .await
        .context("Failed to connect to MongoDB")?;
    let mongo_db = mongo_client.database(&config.database.mongodb.db_name);

    // JWT秘密鍵の読み込み（Configから）
    println!("Loading JWT secret key...");
    let jwt_secret = config.jwt.secret.clone();

    // サービスの構築
    println!("Constructing services...");
    let tag_service = Arc::new(TagService::new(
        Arc::new(TagRepository::new(pg_pool.clone())),
        config.gemini.api_key.clone(),
    ));
    let memo_service = Arc::new(MemoService::new(
        Arc::new(MemoRepository::new(mongo_db.clone())),
        tag_service.clone(),
    ));
    let summary_service = Arc::new(SummaryService::new(
        Arc::new(SummaryRepository::new(mongo_db.clone())),
        Arc::new(MemoRepository::new(mongo_db.clone())),
    ));
    let email_service = Arc::new(services::email_service::EmailService::from_config(
        &config.email.smtp_host,
        config.email.smtp_port,
        &config.email.smtp_username,
        &config.email.smtp_password,
        &config.email.from_email,
        &config.email.from_name,
    )?);
    let verification_store = Arc::new(services::verification_store::VerificationStore::new());
    let email_rate_limiter = Arc::new(services::rate_limiter::EmailRateLimiter::new());
    let auth_rate_limiter = Arc::new(services::rate_limiter::AuthRateLimiter::new());
    let auth_service = Arc::new(AuthService::new(
        Arc::new(repositories::AuthRepository::new(pg_pool.clone())),
        tag_service.clone(),
        jwt_secret.clone(),
        email_service,
        verification_store,
        email_rate_limiter,
    ));

    // AppState の構築
    let state = AppState {
        jwt_decoding_key: auth::create_decoding_key(&jwt_secret),
        auth_service,
        memo_service,
        summary_service,
        tag_service,
        auth_rate_limiter,
        config: Arc::new(config.clone()),
    };
    println!("Constructed AppState");

    // サーバー起動
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("Failed to parse SocketAddr")?;

    server::start_server(addr, state)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
