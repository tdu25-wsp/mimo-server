use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::sync::Arc;

/// 認証コードの目的
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VerificationPurpose {
    Registration,  // ユーザー登録
    PasswordReset, // パスワードリセット
}

/// 認証コードの情報
#[derive(Clone, Debug)]
pub struct VerificationCode {
    pub email: String,
    pub code: String,
    pub purpose: VerificationPurpose,
    pub expires_at: DateTime<Utc>,
    pub attempts: u32, // 試行回数
}

/// 登録トークンの情報
#[derive(Clone, Debug)]
pub struct RegistrationToken {
    pub email: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

/// インメモリストレージ
pub struct VerificationStore {
    // (email, purpose) -> VerificationCode
    verification_codes: Arc<DashMap<(String, VerificationPurpose), VerificationCode>>,
    // token -> RegistrationToken
    registration_tokens: Arc<DashMap<String, RegistrationToken>>,
}

impl VerificationStore {
    pub fn new() -> Self {
        let store = Self {
            verification_codes: Arc::new(DashMap::new()),
            registration_tokens: Arc::new(DashMap::new()),
        };

        // バックグラウンドで定期的に期限切れデータをクリーンアップ
        let codes = store.verification_codes.clone();
        let tokens = store.registration_tokens.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5分ごと
            loop {
                interval.tick().await;
                Self::cleanup_expired(&codes, &tokens);
            }
        });

        store
    }

    /// 認証コードを保存（有効期限: 15分）
    pub fn store_verification_code(
        &self,
        email: String,
        code: String,
        purpose: VerificationPurpose,
    ) {
        let expires_at = Utc::now() + Duration::minutes(15);
        let verification = VerificationCode {
            email: email.clone(),
            code,
            purpose: purpose.clone(),
            expires_at,
            attempts: 0,
        };
        self.verification_codes
            .insert((email, purpose), verification);
    }

    /// 認証コードを検証
    pub fn verify_code(
        &self,
        email: &str,
        code: &str,
        purpose: &VerificationPurpose,
    ) -> Result<bool, String> {
        let key = (email.to_string(), purpose.clone());
        
        // get_mutを使って原子的にチェックと更新を行う
        let mut entry = self
            .verification_codes
            .get_mut(&key)
            .ok_or_else(|| "Verification code not found".to_string())?;

        let now = Utc::now();
        
        // 有効期限チェック
        if now > entry.expires_at {
            drop(entry); // ロックを解放してから削除
            self.verification_codes.remove(&key);
            return Err("Verification code has expired".to_string());
        }

        // 試行回数チェック（5回まで）
        if entry.attempts >= 5 {
            drop(entry);
            self.verification_codes.remove(&key);
            return Err("Too many attempts. Please request a new code".to_string());
        }

        // 試行回数をインクリメント
        entry.attempts += 1;
        let stored_code = entry.code.clone();
        
        // コード照合
        if stored_code == code {
            drop(entry); // ロックを解放してから削除
            self.verification_codes.remove(&key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 登録トークンを保存（有効期限: 15分）
    pub fn store_registration_token(&self, email: String, token: String) {
        let expires_at = Utc::now() + Duration::minutes(15);
        let reg_token = RegistrationToken {
            email: email.clone(),
            token: token.clone(),
            expires_at,
            used: false,
        };
        self.registration_tokens.insert(token, reg_token);
    }

    /// 登録トークンを検証
    pub fn verify_registration_token(&self, token: &str, email: &str) -> Result<(), String> {
        // get_mutを使って原子的にチェックと更新を行う
        let mut entry = self
            .registration_tokens
            .get_mut(token)
            .ok_or_else(|| "Registration token not found".to_string())?;

        let now = Utc::now();
        
        // 有効期限チェック
        if now > entry.expires_at {
            drop(entry); // ロックを解放してから削除
            self.registration_tokens.remove(token);
            return Err("Registration token has expired".to_string());
        }

        // 使用済みチェック（Race Condition防止）
        if entry.used {
            return Err("Registration token has already been used".to_string());
        }

        // メールアドレスチェック
        if entry.email != email {
            return Err("Email address does not match".to_string());
        }

        // 使用済みフラグを立てる（原子的操作）
        entry.used = true;

        Ok(())
    }

    /// 登録トークンを無効化
    pub fn invalidate_registration_token(&self, token: &str) {
        self.registration_tokens.remove(token);
    }

    /// 期限切れデータのクリーンアップ
    fn cleanup_expired(
        codes: &Arc<DashMap<(String, VerificationPurpose), VerificationCode>>,
        tokens: &Arc<DashMap<String, RegistrationToken>>,
    ) {
        let now = Utc::now();

        // 期限切れの認証コードを削除
        codes.retain(|_, v| v.expires_at > now);

        // 期限切れまたは使用済みの登録トークンを削除
        tokens.retain(|_, v| v.expires_at > now && !v.used);
    }

    /// 統計情報取得（デバッグ用）
    pub fn stats(&self) -> (usize, usize) {
        (
            self.verification_codes.len(),
            self.registration_tokens.len(),
        )
    }
}

impl Default for VerificationStore {
    fn default() -> Self {
        Self::new()
    }
}
