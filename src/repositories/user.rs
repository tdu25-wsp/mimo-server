use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn create(&self, user: User) -> Result<User>;
}
