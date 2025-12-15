use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::{AppError, Result},
    repositories::{Tag, TagCreateRequest, TagUpdateRequest, TagRepository, PostgresTagRepository}, // PostgresTagRepositoryをインポート
};

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