use axum::{
    Router,
    routing::{get, post, patch},
    response::{IntoResponse, Json},
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use serde_json::json;
use chrono::{DateTime, Utc};

use crate::repositories::MemoList;

pub fn create_sum_routes() -> Router {
    Router::new()
        .route("/sum/summarize", post(summarize_memo))
        .route("/sum/journaling", get(get_journal))
        .route("/sum/journaling-freq", get(set_frequency))
        .route("/sum/journaling-freq", patch(update_frequency))
}

//// ハンドラ定義

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AISummary {
    summary_id: String,
    user_id: String,
    content: String,

    memo_ids: Vec<String>,
    created_at: String,
    updated_at: String,
    is_auto_generated: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SummaryList {
    summaries: Vec<AISummary>,
}


async fn summarize_memo(jar: CookieJar, memos: Json<MemoList>) -> impl IntoResponse {
    let access_token = jar.get("access_token");
    // TODO: Validate access token

    // メモが空ならAPIを呼ばずにエラーを返す
    if memos.memos.is_empty() {
        return Err(AppError::ValidationError("No memos to summarize".to_string()));
    }

    // AIに読ませるために、複数のメモを一つのテキストに整形する
    let input_text = memos.memos.iter() 
        .map(|memo| format!("- {}", memo.content)) // 各メモをハイフン付きの箇条書き形式に変換
        .collect::<Vec<String>>() // ベクタに収集
        .join("\n"); // 改行で結合して一つの文字列にする
    
    // debug用出力
    println!("AIに送るテキスト:\n{}", input_text);

    // AIに送るプロンプトを作成
    let prompt = format!(
        "以下の箇条書きのメモは、あるユーザーの一日の記録です。これらを統合して、一日の振り返り日記のような自然な文章に要約してください。\n\n[メモ内容]\n{}",
        memos_text
    );

    // APIキーの取得
    let api_key = env::var("OPENAI_API_KEY") //環境変数から取得
        .map_err(|_| AppError::ConfigError("OPENAI_API_KEY is not set".to_string()))?; 

    // OpenAI API呼び出し
    let summary_content = call_openai_api(&api_key, &prompt).await?;

    // レスポンス
    OK(Json(AISummary {
        summary_id: uuid::Uuid::new_v4().to_string(), // ランダムなUUIDを生成
        user_id: "user123".to_string(), //一旦固定値
        content: summary_content, // AIからの要約結果 変数にしました
        memo_ids: memos.memos.iter().map(|m| m.memo_id.clone()).collect(),
        created_at: Utc::now().to_string(),
        updated_at: Utc::now().to_string(),
        is_auto_generated: true,

    }))
}

// OpenAI APIを呼び出す関数
async fn call_openai_api(api_key: &str, prompt: &str) -> Result<String> {
    let client = Client::new(); // reqwestのHTTPクライアントを作成
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions") // URL
        .header("Authorization", format!("Bearer {}", api_key)) // APIキー
        .json(&json!({ //依頼内容
            "model": "gpt-4o-mini", // 使用するモデル 一旦miniにします
            "messages": [
                {"role": "system", "content": "you are a  assistant that summarizes user memos into coherent journal entries."}, // AIの役割
                {"role": "user", "content": prompt} // ユーザープロンプト
            ],
            "max_tokens": 500 // 最大トークン数(返事の長さ)
        }))
        .send()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to send request: {}", e)))?;

    // ステータスコード確認
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::ExternalServiceError(
            format!("OpenAI API error: status={}, body={}", status, error_text)
        ));
    }

    // レスポンスのパース
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse response: {}", e)))?;

    // 要約テキストの抽出
    let content = response_json["choices"]
        .get(0) // Json形式での"choices" リストの 0番目（最初）を見る
        .and_then(|choice| choice["message"]["content"].as_str()) // "message"の"content"を文字列として取得
        .ok_or_else(|| AppError::ExternalServiceError("Invalid response format from OpenAI".to_string()))?
        .to_string();

    Ok(content)
}

async fn get_journal(jar: CookieJar) -> impl IntoResponse {
    Json(SummaryList {
        summaries: vec![], // TODO: Fetch journal entries from DB
    })
}

async fn set_frequency(jar: CookieJar) -> impl IntoResponse {
    // Implementation here
    Json(json!({
        "status": "success",
        "message": "Frequency retrieved successfully"
    }))
}

async  fn update_frequency(jar: CookieJar) -> impl IntoResponse {
    // Implementation here
    Json(json!({
        "status": "success",
        "message": "Frequency updated successfully"
    }))
}
