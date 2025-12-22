use crate::{
    auth::extract_user_id_from_token,
    error::{AppError, Result},
    repositories::{AISummary, SummarizeRequest, SummaryList},
    server::AppState,
};
use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Json},
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
) -> impl IntoResponse {
    let access_token = match jar.get("access_token") {
        Some(token) => token,
        None => {
            return AppError::Unauthorized("Authentication required".to_string()).into_response();
        }
    };

    let authenticated_user_id =
        match extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key) {
            Ok(user_id) => user_id,
            Err(e) => return e.into_response(),
        };

    match state
        .summary_service
        .summarize_and_save(authenticated_user_id, req.memo_ids)
        .await
    {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_summary(
    jar: CookieJar,
    State(state): State<AppState>,
    Path(summary_id): Path<String>,
) -> Result<Json<AISummary>> {
    let access_token = match jar.get("access_token") {
        Some(token) => token,
        None => {
            return Err(AppError::Unauthorized(
                "Authentication required".to_string(),
            ));
        }
    };

    let authenticated_user_id =
        match extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key) {
            Ok(user_id) => user_id,
            Err(e) => {
                return Err(e);
            }
        };

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
) -> impl IntoResponse {
    let access_token = match jar.get("access_token") {
        Some(token) => token,
        None => {
            return AppError::Unauthorized("Authentication required".to_string()).into_response();
        }
    };

    let authenticated_user_id =
        match extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key) {
            Ok(user_id) => user_id,
            Err(e) => return e.into_response(),
        };

    // パスからのuser_idと認証されたユーザーIDが一致するか確認
    if authenticated_user_id != user_id {
        return AppError::Forbidden("Access denied".to_string()).into_response();
    }

    match state.summary_service.get_user_journals(&user_id).await {
        Ok(summaries) => Json(SummaryList { summaries }).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn delete_summary(
    jar: CookieJar,
    State(state): State<AppState>,
    Path(summary_id): Path<String>,
) -> impl IntoResponse {
    let access_token = match jar.get("access_token") {
        Some(token) => token,
        None => {
            return AppError::Unauthorized("Authentication required".to_string()).into_response();
        }
    };

    let authenticated_user_id =
        match extract_user_id_from_token(access_token.value(), &state.jwt_decoding_key) {
            Ok(user_id) => user_id,
            Err(e) => return e.into_response(),
        };

    state
        .summary_service
        .delete_summary(&authenticated_user_id, &summary_id)
        .await
        .into_response()
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
