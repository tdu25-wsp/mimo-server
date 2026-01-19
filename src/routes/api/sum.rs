use crate::{
    error::{AppError, Result, map_error},
    repositories::{AISummary, SummarizeRequest, SummaryList},
    server::AppState,
};
use axum::{
    Router,
    extract::{Path, State},
    response::{Json, Response},
    routing::{delete, get, patch, post},
};
use axum_extra::extract::CookieJar;
use serde_json::json;

pub fn create_sum_routes() -> Router<AppState> {
    Router::new()
        .route("/sum/summarize", post(summarize_memo))
        .route("/sum/{capture}", get(get_summary))
        .route("/sum/list/{capture}", get(get_summaries))
        .route("/sum/{capture}", delete(delete_summary))
        .route("/sum/journaling-freq", get(set_frequency))
        .route("/sum/journaling-freq", patch(update_frequency))
}

async fn summarize_memo(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<SummarizeRequest>,
) -> std::result::Result<Json<AISummary>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    let is_auto_generated = false;
    let summary = state
        .summary_service
        .summarize_and_save(authenticated_user_id, req.memo_ids, is_auto_generated)

        .await.map_err(map_error)?;

    Ok(Json(summary))
}

async fn get_summary(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(summary_id): Path<String>,
) -> std::result::Result<Json<AISummary>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    let summary = state
        .summary_service
        .get_summary_by_id(&authenticated_user_id, &summary_id)
        .await.map_err(map_error)?;

    Ok(Json(summary))
}

async fn get_summaries(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id): Path<String>,
) -> std::result::Result<Json<SummaryList>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(map_error(AppError::Forbidden("Access denied".to_string())));
    }

    let summaries = state.summary_service.get_user_journals(&user_id).await.map_err(map_error)?;
    Ok(Json(SummaryList { summaries }))
}

async fn delete_summary(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(summary_id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, Response> {
    let authenticated_user_id = state.auth_service.extract_and_verify_user_from_access_token(&jar).await?;

    state
        .summary_service
        .delete_summary(&authenticated_user_id, &summary_id)
        .await.map_err(map_error)?;

    Ok(Json(json!({
        "status": "success",
        "message": "Summary deleted successfully"
    })))
}

async fn set_frequency(_jar: CookieJar) -> Result<Json<serde_json::Value>> {
    // TODO: Implementation here
    Ok(Json(json!({
        "status": "success",
        "message": "Frequency retrieved successfully"
    })))
}

async fn update_frequency(_jar: CookieJar) -> Result<Json<serde_json::Value>> {
    // TODO: Implementation here
    Ok(Json(json!({
        "status": "success",
        "message": "Frequency updated successfully"
    })))
}
