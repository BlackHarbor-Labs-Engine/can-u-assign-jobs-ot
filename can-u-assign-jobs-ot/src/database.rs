use sqlx::{SqlitePool, Pool};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database not initialized")]
    NotInitialized,
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

// --- Database Pool Management ---
// Removed global OnceLock. Now we use a factory pattern for test isolation.

/// Initializes and returns a new SqlitePool connected to an in-memory database.
/// This function should be called at the start of each test or service lifecycle.
pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(database_url).await.map_err(DatabaseError::from)?; 

    // Create users table if not exists
    sqlx::query(
        r#"#
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            hashed_password TEXT NOT NULL
        );
        "#
    )
    .execute(&pool)
    .await?;

    // Create jobs table if not exists
    sqlx::query(
        r#"#
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            user_id INTEGER REFERENCES users(id),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

// Removed get_pool() as it relied on the global static.

// --- CRUD Operations ---

pub async fn create_user(pool: &SqlitePool, username: &str, email: &str, hashed_password: &str) -> Result<i64> {
    let result = sqlx::query(
        r#"#
        INSERT INTO users (username, email, hashed_password)
        VALUES (?, ?, ?)
        "#
    )
    .bind(username)
    .bind(email)
    .bind(hashed_password)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<crate::models::User>> {
    let user = sqlx::query_as::<_, crate::models::User>(
        r#"#
        SELECT id, username, email, hashed_password
        FROM users
        WHERE username = ?
        "#
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &SqlitePool, user_id: i64) -> Result<Option<crate::models::User>> {
    let user = sqlx::query_as::<_, crate::models::User>(
        r#"#
        SELECT id, username, email
        FROM users
        WHERE id = ?
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}
