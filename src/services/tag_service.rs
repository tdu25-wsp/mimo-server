use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    error::{AppError, Result},
    repositories::{Tag, TagCreateRequest, TagUpdateRequest, TagRepository},
};

// --- PostgresTagRepository Implementation (Moved from repositories) ---
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

// --- TagService ---

pub struct TagService {
    tag_repo: Arc<PostgresTagRepository>, 
}

impl TagService {
    pub fn new(tag_repo: Arc<PostgresTagRepository>) -> Self {
        Self { tag_repo }
    }

    pub async fn get_user_tags(&self, user_id: &str) -> Result<Vec<Tag>> {
        self.tag_repo.find_by_user_id(user_id).await
    }

    pub async fn create_tag(&self, user_id: String, req: TagCreateRequest) -> Result<Tag> {
        if req.name.trim().is_empty() {
            return Err(AppError::ValidationError("Tag name cannot be empty".into()));
        }

        let now = Utc::now();
        let tag = Tag {
            tag_id: Uuid::new_v4().to_string(),
            user_id,
            name: req.name,
            color_code: req.color_code,
            created_at: now,
            updated_at: now,
        };

        self.tag_repo.create(tag).await
    }

    pub async fn update_tag(&self, tag_id: &str, req: TagUpdateRequest) -> Result<Tag> {
        let mut tag = self.tag_repo.find_by_id(tag_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Tag {} not found", tag_id)))?;

        if let Some(name) = req.name {
            if !name.trim().is_empty() {
                tag.name = name;
            }
        }
        if let Some(color) = req.color_code {
            tag.color_code = color;
        }
        tag.updated_at = Utc::now();

        self.tag_repo.update(tag).await
    }

    pub async fn delete_tag(&self, tag_id: &str) -> Result<()> {
        // 存在確認
        self.tag_repo.find_by_id(tag_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Tag {} not found", tag_id)))?;
            
        self.tag_repo.delete(tag_id).await
    }
}