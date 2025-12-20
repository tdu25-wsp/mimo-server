use chrono::{DateTime, Utc};
use mongodb::action::Update;
use serde::{Deserialize, Serialize};
use crate::error::{Result, AppError};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Tag {
    pub tag_id: String,
    pub user_id: String,
    pub name: String,
    pub color_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct TagList {
    pub tags: Vec<Tag>,
}

#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color_code: String,
}

#[derive(Deserialize)]
pub struct UpdateTagRequest {
    pub name: String,
    pub color_code: String,
}

#[async_trait::async_trait]
pub trait TagHandler: Send + Sync {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Tag>>;
    async fn create(&self, user_id: &str, req: CreateTagRequest) -> Result<Tag>;
    async fn update(&self, user_id: &str, tag_id: &str, req: UpdateTagRequest) -> Result<Tag>;
    async fn delete(&self, user_id: &str, tag_id: &str) -> Result<()>;
}

pub struct TagRepository {
    pub pool: sqlx::PgPool,
}

impl TagRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TagHandler for TagRepository {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT tag_id, user_id, name, color_code, created_at, updated_at FROM tags WHERE user_id = $1",
        )
        .bind(&user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(tags)
    }

    async fn create(&self, user_id: &str, req: CreateTagRequest) -> Result<Tag> {
        let tag = sqlx::query_as::<_, Tag>(
            "INSERT INTO tags (tag_id, user_id, name, color_code, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING tag_id, user_id, name, color_code, created_at, updated_at",
        ).bind(uuid::Uuid::new_v4().to_string())
        .bind(&user_id)
        .bind(&req.name)
        .bind(&req.color_code)
        .bind(&Utc::now())
        .bind(&Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(tag)
    }

    async fn update(&self, user_id: &str, tag_id: &str, req: UpdateTagRequest) -> Result<Tag> {
        // user_id && tag_id で更新対象の特定と権限チェック
        let tag = sqlx::query_as::<_, Tag>(
            "UPDATE tags SET name = $1, color_code = $2, updated_at = $3 WHERE user_id = $4 AND tag_id = $5 RETURNING tag_id, user_id, name, color_code, created_at, updated_at",
        ).bind(&req.name)
        .bind(&req.color_code)
        .bind(&Utc::now())
        .bind(user_id)
        .bind(tag_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(tag)
    }

    async fn delete(&self, user_id: &str, tag_id: &str) -> crate::error::Result<()> {
        // user_id && tag_id で削除対象の特定と権限チェック
        sqlx::query("DELETE FROM tags WHERE user_id = $1 AND tag_id = $2")
            .bind(user_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
