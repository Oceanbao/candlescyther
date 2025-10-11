use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::job::model::{Job, JobStatus, JobType};

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Job execution error: {0}")]
    Execution(String),
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_job(
        &self,
        job_type: JobType,
        payload: serde_json::Value,
    ) -> Result<i64, JobError>;
    async fn get_pending_jobs(&self, limit: usize) -> Result<Vec<Job>, JobError>;
    async fn update_job_status(
        &self,
        job_id: i64,
        status: JobStatus,
        error_message: Option<String>,
    ) -> Result<(), JobError>;
    async fn mark_job_running(&self, job_id: i64) -> Result<(), JobError>;
    async fn mark_job_done(
        &self,
        job_id: i64,
        output: Option<serde_json::Value>,
    ) -> Result<(), JobError>;
}

pub struct SqliteJobRepository {
    pub pool: SqlitePool,
}

impl SqliteJobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepository for SqliteJobRepository {
    async fn create_job(
        &self,
        job_type: JobType,
        payload: serde_json::Value,
    ) -> Result<i64, JobError> {
        let now = chrono::Utc::now().to_string();
        let query = r#"
            INSERT INTO jobs (job_type, job_status, payload, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $4)
        "#;

        let row = sqlx::query(query)
            .bind(job_type)
            .bind(JobStatus::Pending)
            .bind(payload)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(row.last_insert_rowid())
    }

    async fn get_pending_jobs(&self, limit: usize) -> Result<Vec<Job>, JobError> {
        let query = r#"
            SELECT * FROM jobs
            WHERE job_status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
        "#;

        let jobs = sqlx::query_as::<_, Job>(query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        Ok(jobs)
    }

    async fn mark_job_running(&self, job_id: i64) -> Result<(), JobError> {
        let now = chrono::Utc::now().to_string();
        let query = r#"
            UPDATE jobs
            SET job_status = 'running', updated_at = $1
            WHERE id = $2
        "#;
        sqlx::query(query)
            .bind(now)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn mark_job_done(
        &self,
        job_id: i64,
        output: Option<serde_json::Value>,
    ) -> Result<(), JobError> {
        let now = chrono::Utc::now().to_string();
        let query = r#"
            UPDATE jobs
            SET job_status = 'done', updated_at = $1, payload = $2
            WHERE id = $3
        "#;
        sqlx::query(query)
            .bind(now)
            .bind(output)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_job_status(
        &self,
        job_id: i64,
        status: JobStatus,
        error_message: Option<String>,
    ) -> Result<(), JobError> {
        let now = chrono::Utc::now().to_string();
        let query = r#"
            UPDATE jobs
            SET job_status = $2, updated_at = $4, error_message = $3
            WHERE id = $1
        "#;
        sqlx::query(query)
            .bind(job_id)
            .bind(status)
            .bind(error_message)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
