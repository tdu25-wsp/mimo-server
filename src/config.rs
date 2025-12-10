use serde::Deserialize;
use std::fs;
use std::env;
use anyhow::Context;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub postgres: PostgresConfig,
    pub mongodb: MongoDBConfig,
}

#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub connection_url: String,
    pub db_name: String,
}

#[derive(Debug, Deserialize)]
pub struct MongoDBConfig {
    pub connection_uri: String,
    pub db_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // 環境変数から読み込む場合
        if let Ok(postgres_url) = env::var("POSTGRES_CONNECTION_URL") {
            return Ok(Config {
                database: DatabaseConfig {
                    postgres: PostgresConfig {
                        connection_url: postgres_url,
                        db_name: env::var("POSTGRES_DB_NAME")
                            .unwrap_or_else(|_| "mimo_db".to_string()),
                    },
                    mongodb: MongoDBConfig {
                        connection_uri: env::var("MONGODB_CONNECTION_URI")
                            .context("MONGODB_CONNECTION_URI must be set when using env vars")?,
                        db_name: env::var("MONGODB_DB_NAME")
                            .unwrap_or_else(|_| "mimo_db".to_string()),
                    },
                },
                server: ServerConfig {
                    host: env::var("SERVER_HOST")
                        .unwrap_or_else(|_| "0.0.0.0".to_string()),
                    port: env::var("SERVER_PORT")
                        .unwrap_or_else(|_| "5050".to_string())
                        .parse()
                        .unwrap_or(5050),
                },
                logging: LoggingConfig {
                    level: env::var("LOG_LEVEL")
                        .unwrap_or_else(|_| "info".to_string()),
                },
            });
        }

        // Config.tomlから読み込む場合（ローカル開発）
        let config_str = fs::read_to_string("Config.toml")
            .context("Failed to read Config.toml. Use environment variables or provide Config.toml")?;
        
        let config: Config = toml::from_str(&config_str)
            .context("Failed to parse Config.toml")?;
        
        Ok(config)
    }
}
