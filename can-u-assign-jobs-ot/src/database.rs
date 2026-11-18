use sqlx::{SqlitePool, Pool};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database not initialized")]
    NotInitialized,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

static DB_POOL: OnceLock<Pool<sqlx::Sqlite>> = OnceLock::new();

pub async fn init_db(database_url: &str) -> Result<()> {
    let pool = SqlitePool::connect(database_url)
        .await
        .map_err(DatabaseError::from)?;

    // Create users table if not exists
    sqlx::query(
        r#"
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
        r#"
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

    DB_POOL.set(pool).ok().ok_or(DatabaseError::NotInitialized)?;

    Ok(())
}

pub fn get_pool() -> &'static Pool<sqlx::Sqlite> {
    DB_POOL.get().expect("Database not initialized")
}

pub async fn create_user(username: &str, email: &str, hashed_password: &str) -> Result<i64> {
    let pool = get_pool();

    let result = sqlx::query(
        r#"
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

pub async fn get_user_by_username(username: &str) -> Result<Option<sqlx::Row<'static, sqlx::Sqlite>>> {
    let pool = get_pool();

    let user = sqlx::query(
        r#"
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

pub async fn get_user_by_id(user_id: i64) -> Result<Option<sqlx::Row<'static, sqlx::Sqlite>>> {
    let pool = get_pool();

    let user = sqlx::query(
        r#"
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
