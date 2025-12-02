use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memo {
    pub memo_id: String,
    pub content: String,

    pub user_id: String,
    pub tag_id: String,
    pub auto_tag_id: String,
    pub manual_tag_id: Option<String>,

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

// AI Summarization
#[derive(Serialize)]
pub struct AISummary {

}
