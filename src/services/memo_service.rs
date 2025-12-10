use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::{AppError, Result},
    repositories::{Memo, MemoCreateRequest, MongoMemoRepository},
};

pub struct MemoService {
    memo_repo: Arc<MongoMemoRepository>,
}

impl MemoService {
    pub fn new(memo_repo: Arc<MongoMemoRepository>) -> Self {
        Self { memo_repo }
    }

    pub async fn find_by_user(&self, user_id: &str) -> Result<Vec<Memo>> {
        self.memo_repo.find_by_user_id(user_id).await
    }

    pub async fn find_by_id(&self, memo_id: &str) -> Result<Memo> {
        self.memo_repo
            .find_by_id(memo_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Memo {} not found", memo_id)))
    }

    pub async fn create(&self, req: MemoCreateRequest) -> Result<Memo> {
        // バリデーション
        if req.content.trim().is_empty() {
            return Err(AppError::ValidationError("Content cannot be empty".into()));
        }

        let now = Utc::now();
        let memo = Memo {
            memo_id: Uuid::new_v4().to_string(),
            content: req.content,
            user_id: req.user_id,
            tag_id: req.tag_id.unwrap_or_default(),
            auto_tag_id: String::new(),
            manual_tag_id: None,
            share_url_token: None,
            created_at: now,
            updated_at: now,
        };

        self.memo_repo.create(memo).await
    }

    pub async fn delete(&self, memo_id: &str) -> Result<()> {
        // 存在確認
        self.find_by_id(memo_id).await?;
        self.memo_repo.delete(memo_id).await
    }
}
