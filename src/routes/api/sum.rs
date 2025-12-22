use crate::{
    auth::authenticate_from_cookie,
    error::{AppError, Result},
    repositories::{AISummary, SummarizeRequest, SummaryList},
    server::AppState,
};
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
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
) -> Result<Json<AISummary>> {
    let authenticated_user_id = authenticate_from_cookie(&jar, &state.jwt_decoding_key)?;

    let summary = state
        .summary_service
        .summarize_and_save(authenticated_user_id, req.memo_ids)
        .await?;

    Ok(Json(summary))
}

async fn get_summary(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(summary_id): Path<String>,
) -> Result<Json<AISummary>> {
    let authenticated_user_id = authenticate_from_cookie(&jar, &state.jwt_decoding_key)?;

    let summary = state
        .summary_service
        .get_summary_by_id(&authenticated_user_id, &summary_id)
        .await?;

    Ok(Json(summary))
}

async fn get_summaries(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(user_id): Path<String>,
) -> Result<Json<SummaryList>> {
    let authenticated_user_id = authenticate_from_cookie(&jar, &state.jwt_decoding_key)?;

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let summaries = state.summary_service.get_user_journals(&user_id).await?;
    Ok(Json(SummaryList { summaries }))
}

async fn delete_summary(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(summary_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let authenticated_user_id = authenticate_from_cookie(&jar, &state.jwt_decoding_key)?;

    state
        .summary_service
        .delete_summary(&authenticated_user_id, &summary_id)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Summary deleted successfully"
    })))
}

async fn set_frequency(jar: CookieJar) -> Result<Json<serde_json::Value>> {
    // TODO: Implementation here
    Ok(Json(json!({
        "status": "success",
        "message": "Frequency retrieved successfully"
    })))
}

async fn update_frequency(jar: CookieJar) -> Result<Json<serde_json::Value>> {
    // TODO: Implementation here
    Ok(Json(json!({
        "status": "success",
        "message": "Frequency updated successfully"
    })))
}
