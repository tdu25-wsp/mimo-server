use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use crate::error::Result;

// AI-generated summary structure
#[derive(Debug, Clone, Serialize, Deserialize)] //deriving necessary traits
#[serde(rename_all = "camelCase")] // setting serialization format
pub struct AISummary {
    pub summary_id: String,
    pub user_id: String,
    pub content: String,
    pub memo_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_auto_generated: bool,
}

#[async_trait]
// Summary repository trait
pub trait SummaryRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<AISummary>>;
    async fn create(&self, summary: AISummary) -> Result<AISummary>;
}