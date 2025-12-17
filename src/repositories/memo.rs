use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use mongodb::{bson::doc, options::FindOptions};
use crate::error::{Result, AppError};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
pub struct MemoCreateRequest {
    pub user_id: String,
    pub tag_id: Option<String>,
    pub content: String,
}

#[derive(Deserialize)]
pub struct MemoUpdateRequest {
    pub memo_id: String,
    pub tag_id: Option<String>,
    pub content: String,
}

#[async_trait::async_trait]
pub trait MemoHandler: Send + Sync {
    async fn find_by_id(&self, memo_id: &str) -> Result<Option<Memo>>;
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Memo>>;
    async fn create(&self, memo: Memo) -> Result<Memo>;
    async fn update(&self, memo: Memo) -> Result<Memo>;
    async fn delete(&self, memo_id: &str) -> Result<()>;
}

// MemoRepo
pub struct MemoRepository {
    collection: mongodb::Collection<Memo>,
}

impl MemoRepository {
    pub fn new(db: mongodb::Database) -> Self {
        Self { collection: db.collection("memos")  }
    }

}

#[async_trait::async_trait]
impl MemoHandler for MemoRepository {
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Memo>> {
        use futures::stream::TryStreamExt;
        
        self.collection
            .find(doc! { "user_id": user_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .try_collect()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    async fn find_by_id(&self, memo_id: &str) -> Result<Option<Memo>> {
        let memo = self.collection
            .find_one(doc! { "memo_id": memo_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(memo)
    }

    async fn create(&self, memo: Memo) -> Result<Memo> {
        self.collection
            .insert_one(memo.clone())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(memo)
    }

    async fn update(&self, memo: Memo) -> Result<Memo> {
        self.collection
            .replace_one(doc! { "memo_id": &memo.memo_id }, memo.clone())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(memo)
    }

    async fn delete(&self, memo_id: &str) -> Result<()> {
        self.collection
            .delete_one(doc! { "memo_id": memo_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
