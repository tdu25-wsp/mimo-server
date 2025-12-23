use crate::error::{AppError, Result};
use argon2::{
    Argon2, PasswordHash, password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core}
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

pub struct AuthRepository {
    pub pool: sqlx::PgPool,
}

impl AuthRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    // ヘルパーメソッド: パスワードをハッシュ化
    fn hash_password(password: &str) -> Result<String> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand_core::OsRng);
        let hash = PasswordHasher::hash_password(&argon2, password.as_bytes(), &salt)
            .map_err(|e| AppError::HashingError(e.to_string()))?;
        Ok(hash.to_string())
    }

    // ヘルパーメソッド: パスワードを検証
    fn verify_password(password: &str, hash: &str) -> Result<()> {
        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::HashingError(e.to_string()))?;
        argon2.verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|e| AppError::AuthenticationError(e.to_string()))
    }
}

////////
/// 認証関連の構造体
////////
#[derive(Deserialize, Debug)]
pub struct UserLoginRequest {
    pub email: String,
    pub password: String,
}

// 認証メソッド
impl AuthRepository {
    /// パスワードを検証
    pub async fn validate_password(&self, req: UserLoginRequest) -> Result<()> {
        let row = sqlx::query("SELECT password_hash FROM users WHERE email = $1")
            .bind(&req.email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let password_hash = row.get::<String, _>("password_hash");
        Self::verify_password(&req.password, &password_hash)
    }
    
    /// ログアウト処理（JWTを無効化）
    pub async fn logout(&self, jtis: Vec<String>) -> Result<()> {
        for jti in jtis {
            self.revoke_jwt(&jti, chrono::Utc::now()).await?;
        }
        Ok(())
    }

    /// ユーザー登録
    pub async fn register(&self, user: UserCreateRequest) -> Result<UserResponse> {
        self.create_user(user).await
    }
    
    /// パスワードリセット
    pub async fn reset_password(&self, user_id: &str, new_password: &str) -> Result<()> {
        let password_hash = Self::hash_password(new_password)?;

        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE user_id = $3")
            .bind(&password_hash)
            .bind(chrono::Utc::now())
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

////////
/// ユーザー関連の構造体
////////
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

#[derive(Deserialize, Debug)]
pub struct UserCreateRequest {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserUpdateRequest {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub password: Option<String>,
}

// ユーザー管理メソッド
impl AuthRepository {
    /// IDでユーザーを検索
    pub async fn find_user_by_id(&self, user_id: &str) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as::<_, UserResponse>(
            "SELECT user_id, email, display_name, created_at, updated_at, is_active FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(user)
    }

    /// メールアドレスでユーザーを検索
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as::<_, UserResponse>(
            "SELECT user_id, email, display_name, created_at, updated_at, is_active FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// ユーザーを作成
    pub async fn create_user(&self, user: UserCreateRequest) -> Result<UserResponse> {
        let password_hash = Self::hash_password(&user.password)?;
        let now = chrono::Utc::now();
        
        let row = sqlx::query(
            "INSERT INTO users (user_id, email, display_name, password_hash, created_at, updated_at, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING user_id, email, display_name, created_at, updated_at, is_active"
        )
        .bind(&user.user_id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&password_hash)
        .bind(&now)
        .bind(&now)
        .bind(true)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UserResponse {
            user_id: row.get("user_id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        })
    }

    /// ユーザー情報を更新
    pub async fn update_user(&self, user_id: &str, req: UserUpdateRequest) -> Result<UserResponse> {
        let mut query_parts = Vec::new();
        let mut bind_count = 1;

        if req.email.is_some() {
            query_parts.push(format!("email = ${}", bind_count));
            bind_count += 1;
        }
        if req.display_name.is_some() {
            query_parts.push(format!("display_name = ${}", bind_count));
            bind_count += 1;
        }
        if req.password.is_some() {
            query_parts.push(format!("password_hash = ${}", bind_count));
            bind_count += 1;
        }

        if query_parts.is_empty() {
            return self.find_user_by_id(user_id).await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()));
        }

        query_parts.push(format!("updated_at = ${}", bind_count));
        let query_str = format!(
            "UPDATE users SET {} WHERE user_id = ${} RETURNING user_id, email, display_name, created_at, updated_at, is_active",
            query_parts.join(", "),
            bind_count + 1
        );

        let mut query = sqlx::query(&query_str);

        if let Some(email) = req.email {
            query = query.bind(email);
        }
        if let Some(display_name) = req.display_name {
            query = query.bind(display_name);
        }
        if let Some(password) = req.password {
            let password_hash = Self::hash_password(&password)?;
            query = query.bind(password_hash);
        }

        let now = chrono::Utc::now();
        query = query.bind(now).bind(user_id);

        let row = query.fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UserResponse {
            user_id: row.get("user_id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        })
    }

    /// ユーザーを削除（論理削除）
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        sqlx::query("UPDATE users SET is_active = false, updated_at = $1 WHERE user_id = $2")
            .bind(chrono::Utc::now())
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

////////
/// JWT管理メソッド
////////
impl AuthRepository {
    /// JWTを無効化
    pub async fn revoke_jwt(&self, jti: &str, exp: chrono::DateTime<chrono::Utc>) -> Result<()> {
        sqlx::query(
            "INSERT INTO jwt_revocations (jti, expires_at, revoked_at) VALUES ($1, $2, now())",
        )
        .bind(jti)
        .bind(exp)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// JWTが無効化されているか確認
    pub async fn is_jwt_revoked(&self, jti: &str) -> Result<bool> {
        let row =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM jwt_revocations WHERE jti = $1")
                .bind(jti)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.0 > 0)
    }

    /// 期限切れトークンをクリーンアップ
    pub async fn cleanup_expired_tokens(&self) -> Result<()> {
        sqlx::query("DELETE FROM jwt_revocations WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
