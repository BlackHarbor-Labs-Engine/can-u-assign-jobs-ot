use sqlx::{query, Row};
use thiserror::Error;
use crate::database::{get_pool, DatabaseError};

#[derive(Error, Debug)]
pub enum JobRepositoryError {
    #[error("Job not found")]
    NotFound,
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

impl From<sqlx::Error> for JobRepositoryError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => JobRepositoryError::NotFound,
            _ => JobRepositoryError::Database(DatabaseError::Sqlx(err)),
        }
    }
}

pub type Result<T> = std::result::Result<T, JobRepositoryError>;

#[async_trait::async_trait]
pub trait JobRepository {
    async fn create_job(&self, title: &str, description: &str, user_id: i64) -> Result<i64>;
    async fn get_jobs_by_user(&self, user_id: i64) -> Result<Vec<Job>>;
    async fn assign_job_to_user(&self, job_id: i64, user_id: i64) -> Result<()>;
}

pub struct JobRepositoryImpl;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub user_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[async_trait::async_trait]
impl JobRepository for JobRepositoryImpl {
    async fn create_job(&self, title: &str, description: &str, user_id: i64) -> Result<i64> {
        let pool = get_pool();

        let result = query(
            r#"
            INSERT INTO jobs (title, description, user_id)
            VALUES (?, ?, ?)
            "#
        )
        .bind(title)
        .bind(description)
        .bind(Some(user_id))
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn get_jobs_by_user(&self, user_id: i64) -> Result<Vec<Job>> {
        let pool = get_pool();

        let rows = query(
            r#"
            SELECT id, title, description, user_id, created_at, updated_at
            FROM jobs
            WHERE user_id = ?
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let jobs: Vec<Job> = rows.iter().map(|row| Job {
            id: row.get(0),
            title: row.get(1),
            description: row.get(2),
            user_id: row.get::<Option<i64>, _>(3),
            created_at: row.get::<String, _>(4),
            updated_at: row.get::<String, _>(5),
        }).collect();

        Ok(jobs)
    }

    async fn assign_job_to_user(&self, job_id: i64, user_id: i64) -> Result<()> {
        let pool = get_pool();

        query(
            r#"
            UPDATE jobs
            SET user_id = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#
        )
        .bind(Some(user_id))
        .bind(job_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
