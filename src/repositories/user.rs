use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub is_deleted: bool,
}

#[async_trait]
pub trait UserHandler: Send + Sync {
    async fn find_by_id(&self, user_id: &str) -> Result<Option<UserResponse>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<UserResponse>>;
    async fn create(&self, user: User) -> Result<UserResponse>;
    async fn update(&self, user: User) -> Result<UserResponse>;
    async fn delete(&self, user_id: &str) -> Result<()>;
}

pub struct UserRepository{
    pool: sqlx::PgPool
}

impl UserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserHandler for UserRepository {
    async fn find_by_id(&self, user_id: &str) -> Result<Option<UserResponse>> {
        let mut user = sqlx::query_as::<_, UserResponse>(
            "SELECT user_id, email, display_name, created_at, updated_at, is_deleted FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as::<_, UserResponse>(
            "SELECT user_id, email, display_name, created_at, updated_at, is_deleted FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    async fn create(&self, user: User) -> Result<UserResponse> {
        let row = sqlx::query(
            "INSERT INTO users (user_id, email, display_name, password_hash, created_at, updated_at, is_deleted) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING user_id, email, display_name, password_hash, created_at, updated_at, is_deleted"
        )
        .bind(&user.user_id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.password_hash)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .bind(&user.is_deleted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UserResponse {
            user_id: row.get("user_id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_deleted: row.get("is_deleted"),
        })
    }

    async fn update(&self, user: User) -> Result<UserResponse> {
        let row = sqlx::query(
            "UPDATE users SET email = $1, display_name = $2, password_hash = $3, updated_at = $4, is_deleted = $5 WHERE user_id = $6 RETURNING user_id, email, display_name, password_hash, created_at, updated_at, is_deleted"
        )
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.password_hash)
        .bind(&user.updated_at)
        .bind(&user.is_deleted)
        .bind(&user.user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UserResponse {
            user_id: row.get("user_id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_deleted: row.get("is_deleted"),
        })
    }

    async fn delete(&self, user_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE users SET is_deleted = TRUE WHERE user_id = $1"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

