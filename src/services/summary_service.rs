use crate::{
    error::{AppError, Result},
    repositories::{
        AISummary, Memo, MemoHandler, MemoRepository, SummaryRepository, summary::SummaryHandler,
    },
};
use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::{env, sync::Arc};
use uuid::Uuid;

pub struct SummaryService {
    summary_repo: Arc<SummaryRepository>,
    memo_repo: Arc<MemoRepository>,
}

impl SummaryService {
    pub fn new(summary_repo: Arc<SummaryRepository>, memo_repo: Arc<MemoRepository>) -> Self {
        Self {
            summary_repo,
            memo_repo,
        }
    }

    // ユーザーのジャーナル（要約）履歴を取得
    pub async fn get_user_journals(&self, user_id: &str) -> Result<Vec<AISummary>> {
        self.summary_repo.find_by_user_id(user_id).await
    }

    pub async fn get_summary_by_id(&self, user_id: &str, summary_id: &str) -> Result<AISummary> {
        let summary = self
            .summary_repo
            .find_by_id(summary_id)
            .await?
            .ok_or_else(|| AppError::DatabaseError("Summary not found".to_string()))?;

        if summary.user_id != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        Ok(summary)
    }

    // メモのリストを受け取り、要約を生成して保存する
    pub async fn summarize_and_save(
        &self,
        user_id: String,
        memo_ids: Vec<String>,
        is_auto_generated: bool,
    ) -> Result<AISummary> {
        // 0. MemoIDからMemo本体を取得し、user_idでフィルタリング
        let memos = self
            .memo_repo
            .find_by_ids(&memo_ids)
            .await?
            .into_iter()
            .filter(|memo| memo.user_id == user_id)
            .collect::<Vec<Memo>>();

        // メモが空ならAPIを呼ばずにエラーを返す
        if memos.is_empty() {
            return Err(AppError::ValidationError(
                "No memos to summarize".to_string(),
            ));
        }

        // 1. 要約ロジックの実行
        let summary_content = self.call_gemini_api(&memos).await?; // 外部API呼び出し部分

        // 2. DBへの保存データの構築
        let now = Utc::now();
        let summary = AISummary {
            summary_id: Uuid::new_v4().to_string(),
            user_id,
            content: summary_content,
            memo_ids,
            created_at: now,
            updated_at: now,
            is_auto_generated: is_auto_generated,
        };

        // 3. DBへ保存
        self.summary_repo.create(summary).await
    }

    pub async fn delete_summary(&self, user_id: &str, summary_id: &str) -> Result<()> {
        // 削除前に要約の所有者を確認
        self.get_summary_by_id(user_id, summary_id).await?;
        self.summary_repo.delete(summary_id).await
    }

    // Gemini APIを呼び出す関数
    async fn call_gemini_api(&self, memos: &[Memo]) -> Result<String> {
        let input_text = memos
            .iter()
            .map(|memo| format!("- {}", memo.content)) // 各メモをハイフン付きの箇条書き形式に変換
            .collect::<Vec<String>>() // ベクタに収集
            .join("\n"); // 改行で結合して一つの文字列にする

        // debug用出力
        #[cfg(debug_assertions)]
        {
            println!("AIに送るテキスト:\n{}", input_text);
        }

    // AIに送るプロンプトを作成
    let prompt = format!(
        "以下の箇条書きのメモは、あるユーザーの一日の記録です。これらを統合して、一日の振り返り日記のような自然な文章に要約してください。尚、タイトルを先頭に付けることとし、 # タイトル名 の形式で作成した上で改行してください。\n\n[メモ内容]\n{}",
        input_text
    );

        // APIキーの取得
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| AppError::ConfigError("GEMINI_API_KEY is not set".to_string()))?;

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            api_key
        );

        let client = Client::new(); // reqwestのHTTPクライアントを作成
        let response = client
            .post(&url)
            .json(&json!({
                "contents": [{
                    "parts": [{
                        "text": prompt
                    }]
                }]
            }))
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to send request: {}", e))
            })?;

        // ステータスコード確認
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Gemini API error: status={}, body={}",
                status, error_text
            )));
        }

        // レスポンスのパース
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        // 要約テキストの抽出
        let content = response_json["candidates"]
            .get(0)
            .and_then(|c| c["content"]["parts"].get(0)) // 最初の候補の content の parts 配列の最初の要素を取得
            .and_then(|p| p["text"].as_str()) // テキスト部分を文字列として取得
            .ok_or_else(|| {
                AppError::ExternalServiceError("Invalid response format from Gemini".to_string())
            })?
            .to_string();
        Ok(content)
    }
}
