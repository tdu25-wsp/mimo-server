use crate::{
    error::{Result, AppError},
    repositories::{Tag, TagRepository, tag::TagHandler}, // TagHandlerトレイトをインポート
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use reqwest::Client; // HTTPクライアント
use std::env; // 環境変数
use serde_json::json; // JSONマクロ

pub struct TagService {
    tag_repo: Arc<TagRepository>, // MongoTagRepository → TagRepositoryに変更
}

impl TagService {
    pub fn new(tag_repo: Arc<TagRepository>) -> Self {
        Self { tag_repo }
    }

    pub async fn get_tags_by_user(&self, user_id: &str) -> Result<Vec<Tag>> {
        self.tag_repo.find_by_user_id(user_id).await
    }

    pub async fn create_tag(
        &self,
        user_id: String,
        req: CreateTagRequest,
    ) -> Result<Tag> {
        self.tag_repo.create(&user_id, req).await
    }

    pub async fn update_tag(&self, tag: Tag) -> Result<Tag> {
        self.tag_repo.update(tag).await
    }

    pub async fn delete_tag(&self, tag_id: &str) -> Result<()> {
        self.tag_repo.delete(tag_id).await
    }

    /// タグ推薦機能
    pub async fn recommend_tag(&self, user_id: &str, memo_content: &str) -> Result<Option<String>> {
        // ユーザーの全タグを取得
        let tags = self.tag_repo.find_by_user_id(user_id).await?;
        if tags.is_empty() {
            return Ok(None); // タグが1件もない場合
        }

        // タグ名のリストを作成し、内容に合うタグが書かれた命令を作成
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        let tags_str = tag_names.join(", ");

        // プロンプト
        let prompt = format!(
            "以下の[メモ]の内容に最も適したカテゴリを、[タグ候補リスト]から1つだけ選んでください。\n\n[メモ]\n{}\n\n[タグ候補リスト]\n{}\n\n回答はタグ名のみを返してください。適当なタグがない場合は「None」と返してください。",
            memo_content, tags_str
        );

        // APIキーの取得
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| AppError::ConfigError("GEMINI_API_KEY is not set".to_string()))?;

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            api_key
        );
        
        let client = Client::new();
        let response = client // reqwestのHTTPクライアントを作成
            .post(&url) // POSTリクエスト
            .json(&json!({
                "contents": [{
                    "parts": [{ "text": prompt }] // プロンプトをJSON形式で送信
                }]
            }))
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to send request: {}", e)))?;

        // ステータスコードチェック
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(
                format!("Gemini API error in auto-tagging: status={}, body={}", status, error_text)
            ));
        }

        // レスポンス解析
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse response: {}", e)))?;

        let suggested_name = response_json["candidates"] // AIの返答からタグ名を抽出
            .get(0)
            .and_then(|c| c["content"]["parts"].get(0)) // 最初の候補の content の parts 配列の最初の要素を取得
            .and_then(|p| p["text"].as_str()) // テキスト部分を文字列として取得
            .map(|s| s.trim()) // 前後の改行や空白を削除
            .unwrap_or("None");

        // タグ名からIDに変換
        if let Some(tag) = tags.into_iter().find(|t| t.name.trim() == suggested_name) { //DBから取得しておいた tags リストを端から順に見る
            Ok(Some(tag.tag_id))
        } else {
            Ok(None)
        }
    }
}