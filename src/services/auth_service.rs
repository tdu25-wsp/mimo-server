use crate::auth::{
    Role, create_decoding_key, issue_access_token, issue_password_reset_token, issue_refresh_token,
    issue_registration_token, validate_display_name_format, validate_email_format,
    validate_password_format, validate_password_reset_token, validate_registration_token,
    validate_user_id_format,
};
use crate::error::{AppError, Result};
use crate::repositories::auth::{
    AuthRepository, UserCreateRequest, UserLoginRequest, UserResponse, UserUpdateRequest,
};
use crate::repositories::tag::CreateTagRequest;
use crate::services::TagService;
use crate::services::verification_store::VerificationPurpose;
use crate::services::{EmailRateLimiter, EmailService, VerificationStore};
use std::sync::Arc;

pub struct AuthService {
    auth_repo: Arc<AuthRepository>,
    tag_service: Arc<TagService>,
    jwt_secret: String,
    email_service: Arc<EmailService>,
    verification_store: Arc<VerificationStore>,
    rate_limiter: Arc<EmailRateLimiter>,
}

impl AuthService {
    pub fn new(
        auth_repo: Arc<AuthRepository>,
        tag_service: Arc<TagService>,
        jwt_secret: String,
        email_service: Arc<EmailService>,
        verification_store: Arc<VerificationStore>,
        rate_limiter: Arc<EmailRateLimiter>,
    ) -> Self {
        Self {
            auth_repo,
            tag_service,
            jwt_secret,
            email_service,
            verification_store,
            rate_limiter,
        }
    }

    /// ログイン処理
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<(String, String, UserResponse)> {
        let req = UserLoginRequest {
            email: email.clone(),
            password,
        };

        // パスワード検証
        self.auth_repo.validate_password(req).await?;

        // ユーザー情報取得
        let user = self
            .auth_repo
            .find_user_by_email(&email)
            .await?
            .ok_or_else(|| AppError::AuthenticationError("User not found".to_string()))?;

        // アクティブチェック
        if !user.is_active {
            return Err(AppError::AuthenticationError(
                "Account is deactivated".to_string(),
            ));
        }

        // トークン発行
        let roles = vec![
            Role::EditMemo,
            Role::ViewMemo,
            Role::SummarizeMemo,
            Role::EditTag,
            Role::EditAccount,
        ];
        let access_token = issue_access_token(&user.user_id, roles, &self.jwt_secret)?;
        let refresh_token = issue_refresh_token(&user.user_id, &self.jwt_secret)?;

        Ok((access_token, refresh_token, user))
    }

    /// ログアウト処理
    pub async fn logout(&self, jtis: Vec<String>) -> Result<()> {
        // トークンを無効化
        if !jtis.is_empty() {
            self.auth_repo.logout(jtis).await?;
        }

        Ok(())
    }

    /// 汎用: 確認コード送信（登録用またはパスワードリセット用）
    async fn send_verification_code_internal(
        &self,
        email: &str,
        purpose: VerificationPurpose,
        client_ip: Option<&str>,
        check_user_exists: bool, // true: ユーザーが存在すべき, false: ユーザーが存在しないべき
    ) -> Result<()> {
        // レート制限チェック
        self.rate_limiter
            .check_email_limit(email)
            .map_err(|e| AppError::ValidationError(e))?;

        if let Some(ip) = client_ip {
            self.rate_limiter
                .check_ip_limit(ip)
                .map_err(|e| AppError::ValidationError(e))?;
        }

        // ユーザー存在確認
        let user_exists = self.auth_repo.find_user_by_email(email).await?.is_some();

        if check_user_exists && !user_exists {
            return Err(AppError::NotFound("User not found".to_string()));
        }
        if !check_user_exists && user_exists {
            return Err(AppError::ValidationError(
                "Email address is already in use".to_string(),
            ));
        }

        // 確認コード生成（6桁の数字）
        let verification_code = format!("{:06}", rand::random::<u32>() % 1_000_000);

        // インメモリストアに保存
        self.verification_store.store_verification_code(
            email.to_string(),
            verification_code.clone(),
            purpose.clone(),
        );

        // メール送信（目的によって内容を変える）
        match purpose {
            VerificationPurpose::Registration => {
                self.email_service
                    .send_verification_code(email, &verification_code)
                    .await?
            }
            VerificationPurpose::PasswordReset => {
                self.email_service
                    .send_password_reset_code(email, &verification_code)
                    .await?
            }
        }

        Ok(())
    }

    /// ステップ1: メールアドレスで登録開始（認証コード送信）
    pub async fn start_registration(&self, email: String, client_ip: Option<&str>) -> Result<()> {
        // メールアドレスのバリデーション
        validate_email_format(&email)?;

        self.send_verification_code_internal(
            &email,
            VerificationPurpose::Registration,
            client_ip,
            false, // ユーザーが存在しないべき
        )
        .await
    }

    /// ステップ2: メール認証完了後、登録用トークン発行
    pub async fn verify_email_and_issue_registration_token(
        &self,
        email: String,
        code: String,
    ) -> Result<String> {
        // 認証コードを検証
        let is_valid = self
            .verification_store
            .verify_code(&email, &code, &VerificationPurpose::Registration)
            .map_err(|e| AppError::ValidationError(e))?;

        if !is_valid {
            return Err(AppError::ValidationError(
                "Verification code is invalid".to_string(),
            ));
        }

        // 登録用JWTトークン発行（15分間有効）
        let registration_token = issue_registration_token(&email, &self.jwt_secret)?;

        // インメモリストアに保存（使用済みチェック用）
        self.verification_store
            .store_registration_token(email, registration_token.clone());

        Ok(registration_token)
    }

    /// ステップ3: 登録用トークンを検証してユーザー作成
    pub async fn complete_registration(
        &self,
        registration_token: String,
        user: UserCreateRequest,
    ) -> Result<(String, String, UserResponse)> {
        // 入力バリデーション
        validate_email_format(&user.email)?;
        validate_user_id_format(&user.user_id)?;
        validate_password_format(&user.password)?;
        if let Some(ref name) = user.display_name {
            validate_display_name_format(name)?;
        }

        // JWTトークンを検証（有効期限、署名、メールアドレスを確認）
        let key = create_decoding_key(&self.jwt_secret);
        validate_registration_token(&registration_token, &user.email, &key)?;

        // インメモリストアで使用済みチェック
        self.verification_store
            .verify_registration_token(&registration_token, &user.email)
            .map_err(|e| AppError::ValidationError(e))?;

        // メールアドレスの重複チェック（二重チェック）
        if self
            .auth_repo
            .find_user_by_email(&user.email)
            .await?
            .is_some()
        {
            return Err(AppError::ValidationError(
                "Email address is already in use".to_string(),
            ));
        }

        // ユーザー作成
        let user_response = self.auth_repo.register(user).await?;

        // デフォルトタグを作成
        self.create_default_tags(&user_response.user_id).await?;

        // 登録用トークンを無効化
        self.verification_store
            .invalidate_registration_token(&registration_token);

        // 認証トークン発行
        let roles = vec![
            Role::EditMemo,
            Role::ViewMemo,
            Role::SummarizeMemo,
            Role::EditTag,
            Role::EditAccount,
        ];
        let access_token = issue_access_token(&user_response.user_id, roles, &self.jwt_secret)?;
        let refresh_token = issue_refresh_token(&user_response.user_id, &self.jwt_secret)?;

        Ok((access_token, refresh_token, user_response))
    }

    /// レガシー: 即座にユーザー登録（開発用）
    #[deprecated(
        note = "メール認証フローを使用してください: start_registration -> verify_email_and_issue_registration_token -> complete_registration"
    )]
    pub async fn register_immediate(
        &self,
        user: UserCreateRequest,
    ) -> Result<(String, String, UserResponse)> {
        // メールアドレスの重複チェック
        if self
            .auth_repo
            .find_user_by_email(&user.email)
            .await?
            .is_some()
        {
            return Err(AppError::ValidationError(
                "Email address is already in use".to_string(),
            ));
        }

        // ユーザー作成
        let user_response = self.auth_repo.register(user).await?;

        // トークン発行
        let roles = vec![
            Role::EditMemo,
            Role::ViewMemo,
            Role::SummarizeMemo,
            Role::EditTag,
            Role::EditAccount,
        ];
        let access_token = issue_access_token(&user_response.user_id, roles, &self.jwt_secret)?;
        let refresh_token = issue_refresh_token(&user_response.user_id, &self.jwt_secret)?;

        Ok((access_token, refresh_token, user_response))
    }

    /// トークンリフレッシュ
    pub async fn refresh_access_token(&self, user_id: &str) -> Result<String> {
        // ユーザー情報取得
        let user = self
            .auth_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::AuthenticationError("User not found".to_string()))?;

        // アクティブチェック
        if !user.is_active {
            return Err(AppError::AuthenticationError(
                "Account is deactivated".to_string(),
            ));
        }

        // 新しいアクセストークン発行
        let roles = vec![
            Role::EditMemo,
            Role::ViewMemo,
            Role::SummarizeMemo,
            Role::EditTag,
            Role::EditAccount,
        ];
        let access_token = issue_access_token(user_id, roles, &self.jwt_secret)?;

        Ok(access_token)
    }

    /// JTI がrevoke されているか確認
    pub async fn is_token_revoked(&self, jti: &str) -> Result<bool> {
        self.auth_repo.is_jwt_revoked(jti).await
    }

    /// 現在のユーザー情報取得
    pub async fn get_current_user(&self, user_id: &str) -> Result<UserResponse> {
        let user = self
            .auth_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !user.is_active {
            return Err(AppError::AuthenticationError(
                "Account is deactivated".to_string(),
            ));
        }

        Ok(user)
    }

    /// ユーザー情報更新
    pub async fn update_user(&self, user_id: &str, req: UserUpdateRequest) -> Result<UserResponse> {
        // 入力バリデーション
        if let Some(ref email) = req.email {
            validate_email_format(email)?;
            // メールアドレスの重複チェック
            if let Some(existing_user) = self.auth_repo.find_user_by_email(email).await? {
                if existing_user.user_id != user_id {
                    return Err(AppError::ValidationError(
                        "Email address is already in use".to_string(),
                    ));
                }
            }
        }
        if let Some(ref name) = req.display_name {
            validate_display_name_format(name)?;
        }
        if let Some(ref password) = req.password {
            validate_password_format(password)?;
        }

        self.auth_repo.update_user(user_id, req).await
    }

    /// ユーザー削除（論理削除）
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        self.auth_repo.delete_user(user_id).await
    }

    /// パスワードリセット
    pub async fn reset_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // 新しいパスワードのバリデーション
        validate_password_format(new_password)?;

        // 既存パスワードの検証
        let user = self
            .auth_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let login_req = UserLoginRequest {
            email: user.email.clone(),
            password: old_password.to_string(),
        };

        self.auth_repo.validate_password(login_req).await?;

        // 新しいパスワードを設定
        self.auth_repo.reset_password(user_id, new_password).await
    }

    /// ステップ1: パスワードリセット開始（確認コード送信）
    pub async fn forgot_password(&self, email: &str, client_ip: Option<&str>) -> Result<()> {
        // メールアドレスのバリデーション
        validate_email_format(email)?;

        self.send_verification_code_internal(
            email,
            VerificationPurpose::PasswordReset,
            client_ip,
            true, // ユーザーが存在すべき
        )
        .await
    }

    /// ステップ2: 確認コード検証後、リセット用トークン発行
    pub async fn verify_reset_code_and_issue_reset_token(
        &self,
        email: String,
        code: String,
    ) -> Result<String> {
        // 確認コードを検証
        let is_valid = self
            .verification_store
            .verify_code(&email, &code, &VerificationPurpose::PasswordReset)
            .map_err(|e| AppError::ValidationError(e))?;

        if !is_valid {
            return Err(AppError::ValidationError(
                "Verification code is invalid".to_string(),
            ));
        }

        // リセット用JWTトークン発行（30分間有効）
        let reset_token = issue_password_reset_token(&email, &self.jwt_secret)?;

        // インメモリストアに保存（使用済みチェック用）
        self.verification_store
            .store_registration_token(email, reset_token.clone());

        Ok(reset_token)
    }

    /// ステップ3: パスワードリセット完了
    pub async fn complete_password_reset(
        &self,
        reset_token: &str,
        email: &str,
        new_password: &str,
    ) -> Result<()> {
        // 新しいパスワードのバリデーション
        validate_password_format(new_password)?;

        // JWTトークンを検証（有効期限、署名、メールアドレスを確認）
        let key = create_decoding_key(&self.jwt_secret);
        validate_password_reset_token(reset_token, email, &key)?;

        // インメモリストアで使用済みチェック
        self.verification_store
            .verify_registration_token(reset_token, email)
            .map_err(|e| AppError::ValidationError(e))?;

        // ユーザー取得
        let user = self
            .auth_repo
            .find_user_by_email(email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 新しいパスワードを設定
        self.auth_repo
            .reset_password(&user.user_id, new_password)
            .await?;

        // リセットトークンを無効化
        self.verification_store
            .invalidate_registration_token(reset_token);

        Ok(())
    }

    /// 統計情報取得（デバッグ用）
    pub fn get_verification_stats(&self) -> (usize, usize) {
        self.verification_store.stats()
    }

    /// JWT秘密鍵の参照を取得
    pub fn get_secret(&self) -> &str {
        &self.jwt_secret
    }

    /// ユーザー作成時にデフォルトタグを作成
    async fn create_default_tags(&self, user_id: &str) -> Result<()> {
        // デフォルトで作成するタグのリスト
        let default_tags = vec![
            ("仕事", "#3B82F6"),     // 青
            ("生活", "#10B981"),     // 緑
            ("予定", "#EF4444"),     // 赤
            ("アイデア", "#F59E0B"), // オレンジ
            ("趣味", "#8B5CF6"),     // 紫
        ];

        // 各タグを作成
        for (name, color_code) in default_tags {
            let req = CreateTagRequest {
                name: name.to_string(),
                color_code: color_code.to_string(),
            };

            // タグ作成が失敗してもユーザー登録は成功させる（ログのみ）
            if let Err(e) = self.tag_service.create_tag(user_id, req).await {
                eprintln!(
                    "Failed to create default tag '{}' for user {}: {}",
                    name, user_id, e
                );
            }
        }

        Ok(())
    }
}
