mod ai;
mod auth;
mod memos;
mod settings;
mod tags;

pub use ai::create_ai_router;
pub use auth::create_auth_router;
pub use memos::create_memo_router;
pub use settings::create_settings_router;
pub use tags::create_tag_router;
