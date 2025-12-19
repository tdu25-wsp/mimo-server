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
    // AIに読ませるために、複数のメモを一つのテキストに整形する
    let input_text = memos.memos.iter() 
        .map(|memo| format!("- {}", memo.content)) // 各メモをハイフン付きの箇条書き形式に変換
        .collect::<Vec<String>>() // ベクタに収集
        .join("\n"); // 改行で結合して一つの文字列にする
    
    // debug用出力
    println!("AIに送るテキスト:\n{}", input_text);

    Json(AISummary {
        summary_id: "summary123".to_string(),
        user_id: "user123".to_string(),
        content: "This is a summarized content.".to_string(),
        memo_ids: memos.memos.iter().map(|m| m.memo_id.clone()).collect(),
        created_at: Utc::now().to_string(),
        updated_at: Utc::now().to_string(),
        is_auto_generated: true,

    })
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
