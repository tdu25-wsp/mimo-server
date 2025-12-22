use anyhow::Context;
use axum::http::HeaderValue;
use serde::Deserialize;
use std::env;
use std::fs;
use std::str::FromStr;

/// 実行環境を表すenum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl FromStr for Environment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(anyhow::anyhow!("Invalid environment: {}", s)),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

// Serdeでの直列化・非直列化をサポート
impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Environment::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for Environment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Environment::Development => "development",
            Environment::Production => "production",
        };
        serializer.serialize_str(s)
    }
}

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
    #[serde(default)]
    pub env: Environment,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

/// Cookie設定（環境に応じて自動的に決定される）
#[derive(Debug, Clone)]
pub struct CookieConfig {
    pub secure: bool,
    pub same_site: axum_extra::extract::cookie::SameSite,
    pub http_only: bool,
}

impl CookieConfig {
    /// 環境に応じたCookie設定を生成
    pub fn from_environment(env: &Environment) -> Self {
        match env {
            Environment::Production => {
                // プロダクション環境: セキュアな設定
                Self {
                    secure: true,
                    same_site: axum_extra::extract::cookie::SameSite::None,
                    http_only: true,
                }
            }
            Environment::Development => {
                // 開発環境: ローカルテスト向け
                Self {
                    secure: false,
                    same_site: axum_extra::extract::cookie::SameSite::Lax,
                    http_only: true,
                }
            }
        }
    }
}

impl ServerConfig {
    /// 環境に応じたallowed_originsをHeaderValueとして取得
    ///
    /// # Errors
    /// プロダクション環境でallowed_originsが設定されていない場合にエラーを返す
    pub fn get_allowed_origins(
        &self,
        addr: &std::net::SocketAddr,
    ) -> anyhow::Result<Vec<HeaderValue>> {
        let origin_strings = match self.env {
            Environment::Production => {
                // プロダクション環境: 設定ファイルまたは環境変数で指定されたオリジンのみ
                if !self.allowed_origins.is_empty() {
                    self.allowed_origins.clone()
                } else {
                    // プロダクション環境では明示的な指定が必須
                    anyhow::bail!(
                        "Production environment requires explicit ALLOWED_ORIGINS configuration. \
                        Set ALLOWED_ORIGINS environment variable"
                    );
                }
            }
            Environment::Development => {
                // 開発環境: ローカルホスト関連のオリジンを許可
                let mut origins = vec![
                    format!("http://localhost:{}", addr.port()),
                    format!("http://127.0.0.1:{}", addr.port()),
                    "http://localhost:3000".to_string(),
                    format!("http://{}", addr),
                    format!("https://{}", addr),
                ];

                // 設定ファイルで追加のオリジンが指定されている場合は追加
                origins.extend(self.allowed_origins.clone());
                origins
            }
        };

        // String から HeaderValue に変換し、失敗したものはログ出力してスキップ
        let headers: Vec<HeaderValue> = origin_strings
            .into_iter()
            .filter_map(|origin| match origin.parse::<HeaderValue>() {
                Ok(header_value) => {
                    println!("Allowed origin: {}", origin);
                    Some(header_value)
                }
                Err(e) => {
                    eprintln!("Failed to parse origin '{}': {}", origin, e);
                    None
                }
            })
            .collect();

        if headers.is_empty() {
            anyhow::bail!("No valid CORS origins configured");
        }

        Ok(headers)
    }

    /// 環境に応じたCookie設定を取得
    pub fn get_cookie_config(&self) -> CookieConfig {
        CookieConfig::from_environment(&self.env)
    }
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
                    env: env::var("ENVIRONMENT")
                        .ok()
                        .and_then(|s| Environment::from_str(&s).ok())
                        .unwrap_or(Environment::Development),
                    allowed_origins: env::var("ALLOWED_ORIGINS")
                        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                        .unwrap_or_else(|_| Vec::new()),
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
