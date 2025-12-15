use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use sqlx::FromRow; // sqlxを使用
use crate::error::Result;

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

#[derive(Serialize)]
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