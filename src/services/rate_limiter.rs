use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// メール送信用のレートリミッター設定
pub struct EmailRateLimiter {
    // メールアドレスごとのレート制限（1時間に3回まで）
    per_email: Arc<GovernorRateLimiter<String, InMemoryState, DefaultClock>>,
    // IPアドレスごとのレート制限（1時間に10回まで）
    per_ip: Arc<GovernorRateLimiter<String, InMemoryState, DefaultClock>>,
}

impl EmailRateLimiter {
    pub fn new() -> Self {
        // 15分に2回の制限（= 1時間に8回相当だが、バースト防止）
        let email_quota = Quota::within(Duration::from_secs(15 * 60))
            .unwrap()
            .allow_burst(NonZeroU32::new(2).unwrap());
        let per_email = Arc::new(GovernorRateLimiter::keyed(email_quota));

        // 1時間に5回の制限
        let ip_quota = Quota::per_hour(NonZeroU32::new(5).unwrap());
        let per_ip = Arc::new(GovernorRateLimiter::keyed(ip_quota));

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
