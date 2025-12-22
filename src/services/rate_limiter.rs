use governor::{
    clock::{Clock, DefaultClock},
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// メール送信用のレートリミッター設定
pub struct EmailRateLimiter {
    // メールアドレスごとのレート制限（15分間に2回まで）
    per_email: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
    // IPアドレスごとのレート制限（1時間に5回まで）
    per_ip: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
}

/// 認証エンドポイント用のレートリミッター設定
pub struct AuthRateLimiter {
    // IPアドレスごとのレート制限（1時間に20回まで）
    per_ip: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
    // ユーザーIDごとのレート制限（1分間に5回まで）
    per_user: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
}

impl AuthRateLimiter {
    pub fn new() -> Self {
        // 1時間に20回の制限
        let ip_quota = Quota::per_hour(NonZeroU32::new(30).unwrap());
        let per_ip = Arc::new(RateLimiter::dashmap(ip_quota));

        // 1分間に5回の制限（バースト攻撃防止）
        let user_quota = Quota::per_minute(NonZeroU32::new(5).unwrap());
        let per_user = Arc::new(RateLimiter::dashmap(user_quota));

        Self { per_ip, per_user }
    }

    /// Check IP rate limit for authentication endpoints
    pub fn check_ip_limit(&self, ip: &str) -> Result<(), String> {
        match self.per_ip.check_key(&ip.to_string()) {
            Ok(_) => Ok(()),
            Err(negative) => {
                let wait_time = negative.wait_time_from(DefaultClock::default().now());
                let minutes = wait_time.as_secs() / 60;
                let seconds = wait_time.as_secs() % 60;
                
                let time_msg = if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                };
                
                Err(format!(
                    "Too many authentication attempts from this IP. Please try again in {}.",
                    time_msg
                ))
            }
        }
    }

    /// Check user-based rate limit
    pub fn check_user_limit(&self, identifier: &str) -> Result<(), String> {
        match self.per_user.check_key(&identifier.to_string()) {
            Ok(_) => Ok(()),
            Err(negative) => {
                let wait_time = negative.wait_time_from(DefaultClock::default().now());
                let seconds = wait_time.as_secs();
                
                Err(format!(
                    "Too many requests. Please try again in {}s.",
                    seconds
                ))
            }
        }
    }
}

impl Default for AuthRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailRateLimiter {
    pub fn new() -> Self {
        let email_quota = Quota::with_period(Duration::from_secs(15 * 60))
            .unwrap()
            .allow_burst(NonZeroU32::new(3).unwrap());
        let per_email = Arc::new(RateLimiter::dashmap(email_quota));

        let ip_quota = Quota::per_hour(NonZeroU32::new(5).unwrap());
        let per_ip = Arc::new(RateLimiter::dashmap(ip_quota));

        Self { per_email, per_ip }
    }

    /// Check if email sending is allowed
    pub fn check_email_limit(&self, email: &str) -> Result<(), String> {
        match self.per_email.check_key(&email.to_string()) {
            Ok(_) => Ok(()),
            Err(negative) => {
                let wait_time = negative.wait_time_from(DefaultClock::default().now());
                let minutes = wait_time.as_secs() / 60;
                let seconds = wait_time.as_secs() % 60;
                
                let time_msg = if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                };
                
                Err(format!(
                    "Email rate limit exceeded. Please try again in {}.",
                    time_msg
                ))
            }
        }
    }

    /// Check IP rate limit
    pub fn check_ip_limit(&self, ip: &str) -> Result<(), String> {
        match self.per_ip.check_key(&ip.to_string()) {
            Ok(_) => Ok(()),
            Err(negative) => {
                let wait_time = negative.wait_time_from(DefaultClock::default().now());
                let minutes = wait_time.as_secs() / 60;
                let seconds = wait_time.as_secs() % 60;
                
                let time_msg = if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                };
                
                Err(format!(
                    "IP rate limit exceeded. Please try again in {}.",
                    time_msg
                ))
            }
        }
    }
}

impl Default for EmailRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
