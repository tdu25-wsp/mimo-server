mod memo;
mod user;
pub mod summary;
pub mod tag; // tagモジュールを公開

pub use memo::{
    Memo, MemoList, MemoRequest, MemoCreateRequest,
    MemoRepository, MongoMemoRepository,
};
pub use user::{User, UserRepository, PostgresUserRepository};
pub use summary::{AISummary, SummaryRepository, SummaryList, MongoSummaryRepository};
// PostgresTagRepositoryのエクスポートを削除、定義のみエクスポート
pub use tag::{Tag, TagList, TagCreateRequest, TagUpdateRequest, TagRepository};

// MongoDB implementation
use mongodb::{bson::doc, Collection, Database};
use crate::error::{AppError, Result};
use futures::stream::TryStreamExt; // for collecting streams

pub struct MongoMemoRepository {
    collection: Collection<Memo>,
}

impl MongoMemoRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("memos"),
        }
    }

    pub async fn find_by_id(&self, memo_id: &str) -> Result<Option<Memo>> {
        self.collection
            .find_one(doc! { "memoId": memo_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Memo>> {
        use futures::stream::TryStreamExt;

        self.collection
            .find(doc! { "userId": user_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .try_collect()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn create(&self, memo: Memo) -> Result<Memo> {
        self.collection
            .insert_one(&memo)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(memo)
    }

    pub async fn update(&self, memo: Memo) -> Result<Memo> {
        let filter = doc! { "memoId": &memo.memo_id };
        self.collection
            .replace_one(filter, &memo)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(memo)
    }

    pub async fn delete(&self, memo_id: &str) -> Result<()> {
        self.collection
            .delete_one(doc! { "memoId": memo_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

// --- Summary Repository Implementation ---
// MongoDB implementation
pub struct MongoSummaryRepository {
    collection: Collection<AISummary>, // Collection for AISummary
}

impl MongoSummaryRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("summaries"), // Using "summaries" collection
        }
    }

    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<AISummary>> { // Implementing find_by_user_id
        self.collection
            .find(doc! { "userId": user_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))? // Finding documents by userId
            .try_collect()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string())) // Collecting results into a Vec
    }

    pub async fn create(&self, summary: AISummary) -> Result<AISummary> { // Implementing create method
        self.collection
            .insert_one(&summary)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(summary)
    }
}

// PostgreSQL implementation
use sqlx::{PgPool, Row};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, user_id: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT user_id, email, created_at FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| User {
            user_id: r.get("user_id"),
            email: r.get("email"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT user_id, email, created_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| User {
            user_id: r.get("user_id"),
            email: r.get("email"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn create(&self, user: User) -> Result<User> {
        let row = sqlx::query(
            "INSERT INTO users (user_id, email, created_at) VALUES ($1, $2, $3) RETURNING user_id, email, created_at"
        )
        .bind(&user.user_id)
        .bind(&user.email)
        .bind(&user.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(User {
            user_id: row.get("user_id"),
            email: row.get("email"),
            created_at: row.get("created_at"),
        })
    }
}