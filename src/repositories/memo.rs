use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::error::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Memo {
    pub memo_id: String,
    pub content: String,
    pub user_id: String,
    pub tag_id: String,
    pub auto_tag_id: String,
    pub manual_tag_id: Option<Vec<String>>,
    pub share_url_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct MemoList {
    pub memos: Vec<Memo>,
}

#[derive(Deserialize)]
pub struct MemoRequest {
   pub memo_id: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoCreateRequest {
    pub user_id: String,
    pub tag_id: Option<String>,
    pub content: String,
}

// AI Summarization
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AISummary {
    pub user_id: String,
    pub memo_id: Vec<String>,
    pub content: String,
}

#[async_trait::async_trait]
pub trait MemoRepository: Send + Sync {
    async fn find_by_id(&self, memo_id: &str) -> Result<Option<Memo>>;
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Memo>>;
    async fn create(&self, memo: Memo) -> Result<Memo>;
    async fn update(&self, memo: Memo) -> Result<Memo>;
    async fn delete(&self, memo_id: &str) -> Result<()>;
}
