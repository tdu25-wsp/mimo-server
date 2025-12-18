use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
pub struct CreateTagRequest {
    pub name: String,
    pub color_code: String,
}

pub trait TagRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: &str) -> crate::error::Result<Vec<Tag>>;
    async fn create(&self, tag: Tag) -> crate::error::Result<Tag>;
    async fn update(&self, tag: Tag) -> crate::error::Result<Tag>;
    async fn delete(&self, tag_id: &str) -> crate::error::Result<()>;
}
