use crate::{
    error::{AppError, Result},
    repositories::{Memo, MemoCreateRequest, MemoHandler, MemoRepository},
    services::TagService,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct MemoService {
    memo_repo: Arc<MemoRepository>,
    tag_service: Arc<TagService>,
}

impl MemoService {
    pub fn new(memo_repo: Arc<MemoRepository>, tag_service: Arc<TagService>) -> Self {
        Self { memo_repo, tag_service }
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
        validate_memo_content(&req.content)?;

        // タグの自動推薦を試みる
        let auto_tag_id = match self.tag_service.recommend_tag(&req.user_id, &req.content).await {
            Ok(Some(tag_id)) => Some(vec![tag_id]),
            Ok(None) | Err(_) => None, // 推薦失敗時やタグがない場合はNone
        };

        let now = Utc::now();
        let memo = Memo {
            memo_id: Uuid::new_v4().to_string(),
            content: req.content,
            user_id: req.user_id,
            auto_tag_id,
            manual_tag_id: req.tag_id,
            share_url_token: None,
            created_at: now,
            updated_at: now,
        };

        self.memo_repo.create(memo).await
    }

    // メモの更新機能
    pub async fn update_content(&self, memo_id: &str, new_content: String) -> Result<Memo> {
        let mut memo = self.find_by_id(memo_id).await?;

        validate_memo_content(&new_content)?;

        memo.content = new_content;
        memo.updated_at = Utc::now();

        self.memo_repo.update(memo).await
    }
    pub async fn delete(&self, memo_id: &str) -> Result<()> {
        // 存在確認
        self.find_by_id(memo_id).await?;
        self.memo_repo.delete(memo_id).await
    }
}

fn validate_memo_content(content: &str) -> Result<()> {
    let maximum_length = 512; // 最大文字数の例
    if content.trim().is_empty() {
        return Err(AppError::ValidationError("Content cannot be empty".into()));
    }
    if content.len() > maximum_length {
        return Err(AppError::ValidationError(format!("Content cannot exceed {} characters", maximum_length)));
    }
    Ok(())
}