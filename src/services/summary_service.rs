use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::Result,
    repositories::{AISummary, MongoSummaryRepository, Memo},
};

pub struct SummaryService {
    summary_repo: Arc<MongoSummaryRepository>,
}

impl SummaryService {
    pub fn new(summary_repo: Arc<MongoSummaryRepository>) -> Self {
        Self { summary_repo }
    }

    // ユーザーのジャーナル（要約）履歴を取得
    pub async fn get_user_journals(&self, user_id: &str) -> Result<Vec<AISummary>> {
        self.summary_repo.find_by_user_id(user_id).await
    }

    // メモのリストを受け取り、要約を生成して保存する
    pub async fn summarize_and_save(&self, user_id: String, memos: Vec<Memo>) -> Result<AISummary> {
        // 1. 要約ロジックの実行
        let summary_content = self.generate_summary_content(&memos);
        let memo_ids = memos.iter().map(|m| m.memo_id.clone()).collect();

        // 2. DBへの保存データの構築
        let now = Utc::now();
        let summary = AISummary {
            summary_id: Uuid::new_v4().to_string(),
            user_id,
            content: summary_content,
            memo_ids,
            created_at: now,
            updated_at: now,
            is_auto_generated: true,
        };

        // 3. DBへ保存
        self.summary_repo.create(summary).await
    }

    // 要約ロジック（実際にはOpenAI APIなどを呼ぶ箇所）
    fn generate_summary_content(&self, memos: &[Memo]) -> String {
        // Mock Implementation
        // 外部APIを叩けないため、ここでは単純な結合を行います。
        let combined_content: Vec<String> = memos.iter()
            .map(|m| format!("- {}", m.content))
            .collect();
        
        format!(
            "【AI Summary Mock】\nTotal {} memos summarized.\n\nKey Points:\n{}",
            memos.len(),
            combined_content.join("\n")
        )
    }
}