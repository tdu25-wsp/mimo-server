use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use sqlx::{PgPool, FromRow}; // sqlxを使用
use crate::error::{AppError, Result};

// FromRowをderiveして、DBの行から構造体へ自動マッピング
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)] 
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub tag_id: String,
    pub user_id: String,
    pub name: String,
    pub color_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct TagList {
    pub tags: Vec<Tag>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagCreateRequest {
    pub name: String,
    pub color_code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdateRequest {
    pub name: Option<String>,
    pub color_code: Option<String>,
}

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Tag>>;
    async fn find_by_id(&self, tag_id: &str) -> Result<Option<Tag>>;
    async fn create(&self, tag: Tag) -> Result<Tag>;
    async fn update(&self, tag: Tag) -> Result<Tag>;
    async fn delete(&self, tag_id: &str) -> Result<()>;
}

// PostgreSQL Implementation
pub struct PostgresTagRepository {
    pool: PgPool,
}

impl PostgresTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for PostgresTagRepository {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT * FROM tags WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(tags)
    }

    async fn find_by_id(&self, tag_id: &str) -> Result<Option<Tag>> {
        let tag = sqlx::query_as::<_, Tag>(
            "SELECT * FROM tags WHERE tag_id = $1"
        )
        .bind(tag_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(tag)
    }

    async fn create(&self, tag: Tag) -> Result<Tag> {
        sqlx::query(
            "INSERT INTO tags (tag_id, user_id, name, color_code, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&tag.tag_id)
        .bind(&tag.user_id)
        .bind(&tag.name)
        .bind(&tag.color_code)
        .bind(tag.created_at)
        .bind(tag.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(tag)
    }

    async fn update(&self, tag: Tag) -> Result<Tag> {
        sqlx::query(
            "UPDATE tags SET name = $1, color_code = $2, updated_at = $3 WHERE tag_id = $4"
        )
        .bind(&tag.name)
        .bind(&tag.color_code)
        .bind(tag.updated_at)
        .bind(&tag.tag_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(tag)
    }

    async fn delete(&self, tag_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM tags WHERE tag_id = $1")
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Tag {} not found", tag_id)));
        }

        Ok(())
    }
}