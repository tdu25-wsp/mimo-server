use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::Result,
    repositories::{AISummary, SummaryRepository, summary::SummaryHandler, MemoRepository, MemoHandler, Memo},
};

pub struct SummaryService {
    summary_repo: Arc<SummaryRepository>,
    memo_repo: Arc<MemoRepository>,
}

impl SummaryService {
    // コンストラクタで memo_repo も受け取る
    pub fn new(
        summary_repo: Arc<SummaryRepository>, 
        memo_repo: Arc<MemoRepository> 
    ) -> Self {
        Self { summary_repo, memo_repo }
    }

    // ユーザーのジャーナル（要約）履歴を取得
    pub async fn get_user_journals(&self, user_id: &str) -> Result<Vec<AISummary>> {
        self.summary_repo.find_by_user_id(user_id).await
    }

    // メモのリストを受け取り、要約を生成して保存する
    pub async fn summarize_and_save(&self, user_id: String, memo_ids: Vec<String>) -> Result<AISummary> {
        // 0. MemoIDからMemo本体を取得
        let mut memos = Vec::new();
        for id in &memo_ids {
            // 見つかったメモだけを処理対象とする
            if let Ok(Some(memo)) = self.memo_repo.find_by_id(id).await {
                if memo.user_id == user_id {
                    memos.push(memo);
                }
            }
        }
        // 1. 要約ロジックの実行
        let summary_content = self.generate_summary_content(&memos);

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
        // 外部APIを叩けないため、ここでは単純な結合
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
