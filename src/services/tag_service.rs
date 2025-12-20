use crate::{
    error::Result,
    repositories::{Tag, TagRepository, TagHandler, CreateTagRequest, UpdateTagRequest},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct TagService {
    tag_repo: Arc<TagRepository>,
}

impl TagService {
    pub fn new(tag_repo: Arc<TagRepository>) -> Self {
        Self { tag_repo }
    }

    pub async fn get_tags_by_user(&self, user_id: &str) -> Result<Vec<Tag>> {
        self.tag_repo.find_by_user_id(user_id).await
    }

    pub async fn create_tag(
        &self,
        user_id: &str,
        req: CreateTagRequest,
    ) -> Result<Tag> {
        self.tag_repo.create(user_id, req).await
    }

    pub async fn update_tag(&self, user_id: &str, tag_id: &str, req: UpdateTagRequest) -> Result<Tag> {
        self.tag_repo.update(user_id, tag_id, req).await
    }

    pub async fn delete_tag(&self,user_id: &str, tag_id: &str) -> Result<()> {
        self.tag_repo.delete(user_id, tag_id).await
    }
}
