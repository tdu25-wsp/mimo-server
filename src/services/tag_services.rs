use crate::{
    error::Result,
    repositories::{Tag, TagRepository},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct TagService {
    tag_repo: Arc<MongoTagRepository>,
}

impl TagService {
    pub fn new(tag_repo: Arc<MongoTagRepository>) -> Self {
        Self { tag_repo }
    }

    pub async fn get_tags_by_user(&self, user_id: &str) -> Result<Vec<Tag>> {
        self.tag_repo.find_by_user_id(user_id).await
    }

    pub async fn create_tag(
        &self,
        user_id: String,
        name: String,
        color_code: String,
    ) -> Result<Tag> {
        let now = Utc::now();
        let tag = Tag {
            tag_id: Uuid::new_v4().to_string(),
            user_id,
            name,
            color_code,
            created_at: now,
            updated_at: now,
        };
        self.tag_repo.create(tag).await
    }

    pub async fn update_tag(&self, tag: Tag) -> Result<Tag> {
        self.tag_repo.update(tag).await
    }

    pub async fn delete_tag(&self, tag_id: &str) -> Result<()> {
        self.tag_repo.delete(tag_id).await
    }
}
