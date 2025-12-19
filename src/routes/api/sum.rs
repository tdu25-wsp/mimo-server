use crate::{
    repositories::{AISummary, SummaryCreateRequest, SummaryList},
    server::AppState,
};
use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Json},
    routing::{get, patch, post},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize; 
use serde_json::json;

pub fn create_sum_routes() -> Router<AppState> {
    Router::new()
        .route("/sum/summarize", post(summarize_memo))
        .route("/sum/{capture}", get(get_summaries))
        .route("/sum/journaling-freq", get(set_frequency))
        .route("/sum/journaling-freq", patch(update_frequency))
}

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub memo_ids: Vec<String>,
}

async fn summarize_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<SummarizeRequest>,
) -> impl IntoResponse {
    let _access_token = jar.get("access_token");
    // TODO: Validate access token
    // TODO: summarize_memos
    let user_id = "user123".to_string();

    // memo_ids を渡してサービスを呼び出す
    match state
        .summary_service
        .summarize_and_save(user_id, req.memo_ids)
        .await
    {
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
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let _access_token = jar.get("access_token");
    match state.summary_service.get_user_journals(&user_id).await {
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
