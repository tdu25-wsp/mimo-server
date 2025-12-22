use anyhow::Context;
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub jwt: JwtConfig,
    pub email: EmailConfig,
    pub gemini: GeminiConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub postgres: PostgresConfig,
    pub mongodb: MongoDBConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    pub connection_url: String,
    pub db_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MongoDBConfig {
    pub connection_uri: String,
    pub db_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
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
                    host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                    port: env::var("SERVER_PORT")
                        .unwrap_or_else(|_| "5050".to_string())
                        .parse()
                        .unwrap_or(5050),
                },
                logging: LoggingConfig {
                    level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                },
                jwt: JwtConfig {
                    secret: env::var("JWT_SECRET")
                        .context("JWT_SECRET must be set when using env vars")?,
                },
                email: EmailConfig {
                    smtp_host: env::var("SMTP_HOST")
                        .context("SMTP_HOST must be set when using env vars")?,
                    smtp_port: env::var("SMTP_PORT")
                        .unwrap_or_else(|_| "587".to_string())
                        .parse()
                        .context("Invalid SMTP_PORT")?,
                    smtp_username: env::var("SMTP_USERNAME")
                        .context("SMTP_USERNAME must be set when using env vars")?,
                    smtp_password: env::var("SMTP_PASSWORD")
                        .context("SMTP_PASSWORD must be set when using env vars")?,
                    from_email: env::var("SMTP_FROM_EMAIL")
                        .unwrap_or_else(|_e| env::var("SMTP_USERNAME").unwrap_or_default()),
                    from_name: env::var("SMTP_FROM_NAME")
                        .unwrap_or_else(|_| "Mimo Server".to_string()),
                },
                gemini: GeminiConfig {
                    api_key: env::var("GEMINI_API_KEY").unwrap_or_else(|_| String::new()),
                },
            });
        }

        // Config.tomlから読み込む場合（ローカル開発）
        let config_str = fs::read_to_string("Config.toml").context(
            "Failed to read Config.toml. Use environment variables or provide Config.toml",
        )?;

        let mut config: Config =
            toml::from_str(&config_str).context("Failed to parse Config.toml")?;

        // 環境変数があれば優先する
        if let Ok(secret) = env::var("JWT_SECRET") {
            config.jwt.secret = secret;
        }
        if let Ok(host) = env::var("SMTP_HOST") {
            config.email.smtp_host = host;
        }
        if let Ok(port) = env::var("SMTP_PORT") {
            if let Ok(port_num) = port.parse() {
                config.email.smtp_port = port_num;
            }
        }
        if let Ok(username) = env::var("SMTP_USERNAME") {
            config.email.smtp_username = username;
        }
        if let Ok(password) = env::var("SMTP_PASSWORD") {
            config.email.smtp_password = password;
        }
        if let Ok(from_email) = env::var("SMTP_FROM_EMAIL") {
            config.email.from_email = from_email;
        }
        if let Ok(from_name) = env::var("SMTP_FROM_NAME") {
            config.email.from_name = from_name;
        }
        if let Ok(api_key) = env::var("GEMINI_API_KEY") {
            config.gemini.api_key = api_key;
        }

        Ok(config)
    }
}
