use axum::{
    Router,
    extract::Path,
    response::{IntoResponse, Json},
    routing::{get, patch, post},
};
use axum_extra::extract::CookieJar;
use serde_json::json;
use std::sync::Arc;

// repositoriesから共通の構造体を使用
use crate::{
    repositories::{AISummary, SummaryList},
    services::SummaryService,
};

pub fn create_sum_routes(service: Arc<SummaryService>) -> Router {
    Router::new()
        .route("/sum/summarize", {
            let service = service.clone();
            post(move |jar, json| summarize_memo(jar, service.clone(), json))
        })
        .route("/sum/{capture}", {
            let service = service.clone();
            get(move |jar, path| get_summaries(jar, service.clone(), path))
        })
        .route("/sum/journaling-freq", get(set_frequency))
        .route("/sum/journaling-freq", patch(update_frequency))
}

// リクエストボディ用の構造体を定義
#[derive(serde::Deserialize)]
struct SummarizeRequest {
    memo_ids: Vec<String>,
}

async fn summarize_memo(
    jar: CookieJar,
    service: Arc<SummaryService>,
    Json(req): Json<SummarizeRequest>,
) -> impl IntoResponse {
    let _access_token = jar.get("access_token");
    // TODO: Validate access token
    // TODO: summarize_memos
    let user_id = "user_123".to_string();

    // memo_ids を渡してサービスを呼び出す
    match service.summarize_and_save(user_id, req.memo_ids).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => e.into_response(),
    }
}

//async fn get_summary(
//    jar: CookieJar,
//    service: Arc<SummaryService>,
//    Path(summary_id): Path<String>,
//) -> impl IntoResponse {
//    let _access_token = jar.get("access_token");
//    match service.get_summary_by_id(&summary_id).await {
//        Ok(summary) => Json(summary).into_response(),
//        Err(e) => e.into_response(),
//    }
//}

async fn get_summaries(
    jar: CookieJar,
    service: Arc<SummaryService>,
    path: Path<String>,
) -> impl IntoResponse {
    let _access_token = jar.get("access_token");
    let user_id = path;
    match service.get_user_journals(&user_id).await {
        // repositories::SummaryList を使用
        Ok(summaries) => Json(SummaryList { summaries }).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn set_frequency(jar: CookieJar) -> impl IntoResponse {
    // Implementation here
    Json(json!({
        "status": "success",
        "message": "Frequency retrieved successfully"
    }))
}

async fn update_frequency(jar: CookieJar) -> impl IntoResponse {
    // Implementation here
    Json(json!({
        "status": "success",
        "message": "Frequency updated successfully"
    }))
}
