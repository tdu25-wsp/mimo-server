use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[async_trait::async_trait]
pub trait TagHandler: Send + Sync {
    async fn find_by_user_id(&self, user_id: &str) -> crate::error::Result<Vec<Tag>>;
    async fn create(&self, tag: Tag) -> crate::error::Result<Tag>;
    async fn update(&self, tag: Tag) -> crate::error::Result<Tag>;
    async fn delete(&self, tag_id: &str) -> crate::error::Result<()>;
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
    async fn find_by_user_id(&self, user_id: &str) -> crate::error::Result<Vec<Tag>> {
        let rows = sqlx::query_as::<_, Tag>(
            "SELECT tag_id, user_id, name, color_code, created_at, updated_at FROM tags WHERE user_id = $1",
        )
        .bind(&user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows)
    }

    async fn create(&self, tag: Tag) -> crate::error::Result<Tag> {
        let row = sqlx::query_as::<_, Tag>(
            "INSERT INTO tags (tag_id, user_id, name, color_code, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING tag_id, user_id, name, color_code, created_at, updated_at",
        ).bind(&tag.tag_id)
        .bind(&tag.user_id)
        .bind(&tag.name)
        .bind(&tag.color_code)
        .bind(&tag.created_at)
        .bind(&tag.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Tag {
            tag_id: row.tag_id,
            user_id: row.user_id,
            name: row.name,
            color_code: row.color_code,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn update(&self, tag: Tag) -> crate::error::Result<Tag> {
        let row = sqlx::query_as::<_, Tag>(
            "UPDATE tags SET name = $1, color_code = $2, updated_at = $3 WHERE tag_id = $4 RETURNING tag_id, user_id, name, color_code, created_at, updated_at",
        ).bind(&tag.name)
        .bind(&tag.color_code)
        .bind(&tag.updated_at)
        .bind(&tag.tag_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Tag {
            tag_id: row.tag_id,
            user_id: row.user_id,
            name: row.name,
            color_code: row.color_code,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete(&self, tag_id: &str) -> crate::error::Result<()> {
        sqlx::query("DELETE FROM tags WHERE tag_id = $1")
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
