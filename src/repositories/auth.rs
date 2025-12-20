use crate::error::{AppError, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core},
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
}

/////////
/// User
/////////
#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub is_deleted: bool,
}

#[derive(Deserialize, Debug)]
pub struct UserCreateRequest {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserUpdateRequest {
    pub user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub password: Option<String>,
}

#[async_trait::async_trait]
pub trait UserHandler: Send + Sync {
    async fn find_by_id(&self, user_id: &str) -> Result<Option<UserResponse>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<UserResponse>>;
    async fn create(&self, user: UserCreateRequest) -> Result<UserResponse>;
    async fn update(&self, user: UserUpdateRequest) -> Result<UserResponse>;
    async fn delete(&self, user_id: &str) -> Result<()>;
}

#[async_trait::async_trait]
impl UserHandler for AuthRepository {
    async fn find_by_id(&self, user_id: &str) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as::<_, UserResponse>(
            "SELECT user_id, email, display_name, created_at, updated_at, is_deleted FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as::<_, UserResponse>(
            "SELECT user_id, email, display_name, created_at, updated_at, is_deleted FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    async fn create(&self, user: UserCreateRequest) -> Result<UserResponse> {
        let argon2 = Argon2::default();

        let user_id = uuid::Uuid::new_v4().to_string();
        let salt = SaltString::generate(&mut rand_core::OsRng);
        let password_hash = PasswordHasher::hash_password(&argon2, user.password.as_bytes(), &salt)
            .map_err(|e| AppError::HashingError(e.to_string()))?
            .to_string();
        let created_at = chrono::Utc::now();
        let updated_at = created_at;
        let is_deleted = false;
        let row = sqlx::query(
            "INSERT INTO users (user_id, email, display_name, password_hash, created_at, updated_at, is_deleted) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING user_id, email, display_name, password_hash, created_at, updated_at, is_deleted"
        )
        .bind(&user_id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&password_hash)
        .bind(&created_at)
        .bind(&updated_at)
        .bind(&is_deleted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UserResponse {
            user_id: row.get("user_id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_deleted: row.get("is_deleted"),
        })
    }

    async fn update(&self, user: UserUpdateRequest) -> Result<UserResponse> {
        let row = if Some(&user.password).is_some() {
            // パスワード変更が必要な場合
            let argon2 = Argon2::default();
            let salt = SaltString::generate(&mut rand_core::OsRng);
            let password_hash =
                PasswordHasher::hash_password(&argon2, user.password.unwrap().as_bytes(), &salt)
                    .map_err(|e| AppError::HashingError(e.to_string()))?
                    .to_string();

            sqlx::query(
                "UPDATE users SET email = $1, display_name = $2, password_hash = $3, updated_at = $4 WHERE user_id = $5 RETURNING user_id, email, display_name, password_hash, created_at, updated_at, is_deleted"
            )
            .bind(&user.email)
            .bind(&user.display_name)
            .bind(&password_hash)
            .bind(&chrono::Utc::now())
            .bind(&user.user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        } else {
            // パスワード変更が不要な場合
            sqlx::query(
                "UPDATE users SET email = $1, display_name = $2, updated_at = $3 WHERE user_id = $4 RETURNING user_id, email, display_name, password_hash, created_at, updated_at, is_deleted"
            )
            .bind(&user.email)
            .bind(&user.display_name)
            .bind(&chrono::Utc::now())
            .bind(&user.user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        };

        Ok(UserResponse {
            user_id: row.get("user_id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_deleted: row.get("is_deleted"),
        })
    }

    async fn delete(&self, user_id: &str) -> Result<()> {
        sqlx::query("UPDATE users SET is_deleted = TRUE WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

/////////
/// JWT Revocation
/// /////
#[async_trait::async_trait]
pub trait JWTHandler: Send + Sync {
    async fn revoke(&self, jti: &str, exp: &str) -> Result<()>;
    async fn is_revoked(&self, jti: &str) -> Result<bool>;
    async fn cleanup(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl JWTHandler for AuthRepository {
    async fn revoke(&self, jti: &str, exp: &str) -> Result<()> {
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

    async fn is_revoked(&self, jti: &str) -> Result<bool> {
        let row =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM jwt_revocations WHERE jti = $1")
                .bind(jti)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.0 > 0)
    }

    async fn cleanup(&self) -> Result<()> {
        sqlx::query("DELETE FROM jwt_revocations WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
