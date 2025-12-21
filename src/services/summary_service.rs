use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use reqwest::Client; // HTTPクライアント用
use std::env; // 環境変数取得用
use serde_json::json; // JSON構築用
use crate::{
    error::{Result, AppError}, // エラーハンドリング用
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

    // メモが空ならAPIを呼ばずにエラーを返す
    if memos.is_empty() {
        return Err(AppError::ValidationError("No memos to summarize".to_string()));
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
            is_auto_generated: true,
        };

        // 3. DBへ保存
        self.summary_repo.create(summary).await
    }

    // Gemini APIを呼び出す関数
async fn call_gemini_api(&self, memos: &[Memo]) -> Result<String> {
     // AIに読ませるために、複数のメモを一つのテキストに整形する
    let input_text = memos.iter() 
        .map(|memo| format!("- {}", memo.content)) // 各メモをハイフン付きの箇条書き形式に変換
        .collect::<Vec<String>>() // ベクタに収集
        .join("\n"); // 改行で結合して一つの文字列にする
    
    // debug用出力
    println!("AIに送るテキスト:\n{}", input_text);

    // AIに送るプロンプトを作成
    let prompt = format!(
        "以下の箇条書きのメモは、あるユーザーの一日の記録です。これらを統合して、一日の振り返り日記のような自然な文章に要約してください。\n\n[メモ内容]\n{}",
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
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to send request: {}", e)))?;

    // ステータスコード確認
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::ExternalServiceError(
            format!("Gemini API error: status={}, body={}", status, error_text)
        ));
    }

    // レスポンスのパース
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse response: {}", e)))?;

    // 要約テキストの抽出
    let content = response_json["candidates"]
            .get(0) 
            .and_then(|c| c["content"]["parts"].get(0)) // 最初の候補の content の parts 配列の最初の要素を取得
            .and_then(|p| p["text"].as_str())           // テキスト部分を文字列として取得
            .ok_or_else(|| AppError::ExternalServiceError("Invalid response format from Gemini".to_string()))?
            .to_string();
    Ok(content)
    }
}
