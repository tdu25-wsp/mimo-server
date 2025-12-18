use crate::error::{AppError, Result};

#[async_trait::async_trait]
pub trait JWTHandler: Send + Sync {
    async fn revoke(&self, jti: &str, exp: &str) -> Result<()>;
    async fn is_revoked(&self, jti: &str) -> Result<bool>;
    async fn celanup(&self) -> Result<()>;
}

pub struct RevocationRepository {
    pool: sqlx::PgPool,
}

impl RevocationRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JWTHandler for RevocationRepository {
    async fn revoke(&self, jti: &str, exp: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO jwt_revocations (jti, expires_at, revoked_at) VALUES ($1, $2, now())"
        )
        .bind(jti)
        .bind(exp)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn is_revoked(&self, jti: &str) -> Result<bool> {
        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM jwt_revocations WHERE jti = $1"
        )
        .bind(jti)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.0 > 0)
    }
    
    async fn celanup(&self) -> Result<()> {
        sqlx::query(
            "DELETE FROM jwt_revocations WHERE expires_at < now()"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
