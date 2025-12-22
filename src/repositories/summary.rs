use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// AI-generated summary structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISummary {
    pub summary_id: String,
    pub user_id: String,
    pub content: String,
    pub memo_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_auto_generated: bool,
}

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub memo_ids: Vec<String>,
}

// Wrapper for a list of AI summaries
#[derive(Serialize, Deserialize)]
pub struct SummaryList {
    pub summaries: Vec<AISummary>,
}

#[async_trait]
// Summary repository trait
pub trait SummaryHandler: Send + Sync {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<AISummary>>;
    async fn find_by_id(&self, summary_id: &str) -> Result<Option<AISummary>>;
    async fn create(&self, summary: AISummary) -> Result<AISummary>;
    async fn delete(&self, summary_id: &str) -> Result<()>;
}

pub struct SummaryRepository {
    collection: mongodb::Collection<AISummary>,
}

impl SummaryRepository {
    pub fn new(db: mongodb::Database) -> Self {
        Self {
            collection: db.collection("summaries"),
        }
    }
}

#[async_trait]
impl SummaryHandler for SummaryRepository {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<AISummary>> {
        use futures::stream::TryStreamExt;
        self.collection
            .find(mongodb::bson::doc! { "user_id": user_id })
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?
            .try_collect()
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }

    async fn find_by_id(&self, summary_id: &str) -> Result<Option<AISummary>> {
        self.collection
            .find_one(mongodb::bson::doc! { "summary_id": summary_id })
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }

    async fn create(&self, summary: AISummary) -> Result<AISummary> {
        self.collection
            .insert_one(&summary)
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;
        Ok(summary)
    }

    async fn delete(&self, summary_id: &str) -> Result<()> {
        self.collection
            .delete_one(mongodb::bson::doc! { "summary_id": summary_id })
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
