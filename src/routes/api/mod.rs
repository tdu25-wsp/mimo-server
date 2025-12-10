use axum::Router;
use std::sync::Arc;

mod auth;
mod memo;
mod settings;
mod sum;
mod tags;

use auth::create_auth_routes;
use memo::create_memo_routes;
use settings::create_settings_routes;
use sum::create_sum_routes;
use tags::create_tags_routes;

use crate::services::MemoService;

pub fn create_api_routes(memo_service: Arc<MemoService>) -> Router {
    Router::new()
        .merge(create_auth_routes())
        .merge(create_sum_routes())
        .merge(create_memo_routes(memo_service))
        .merge(create_tags_routes())
        .merge(create_settings_routes())
}
