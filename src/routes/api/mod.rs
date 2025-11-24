use axum::Router;

// 各サブモジュールの定義
mod ai;
mod auth;
mod memos;
mod settings;
mod tags;

// 内部実装がない場合はスタブが必要ですが、まずはルーター作成関数を定義
// (各サブモジュール内に create_xxx_router が存在すると仮定、なければ空実装を作る必要があります)
use ai::create_ai_routes;
use auth::create_auth_routes;
use memos::create_memos_routes;
use settings::create_settings_routes;
use tags::create_tags_routes;

pub async fn create_api_routes() -> Router {
    Router::new()
        .nest("/auth", create_auth_routes())
        .nest("/memos", create_memos_routes())
        .nest("/settings", create_settings_routes())
        .nest("/tags", create_tags_routes())
        .nest("/ai", create_ai_routes())
}