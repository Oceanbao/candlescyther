use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::application::{
    model::{Job, JobStatus},
    repository::JobRepository,
    runner::RunnerError,
};

#[derive(Clone)]
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
    async fn create_jobs(&self, jobs: Vec<Job>) -> Result<(), RunnerError> {
        let tx = self.pool.begin().await?;

        for job in jobs {
            sqlx::query!(
                r#"
            INSERT INTO jobs (job_type, job_status, payload, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
                job.job_type,
                job.job_status,
                job.payload,
                job.created_at,
                job.updated_at,
            )
            .execute(&self.pool)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn get_pending_jobs(&self, limit: usize) -> Result<Vec<Job>, RunnerError> {
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

    async fn mark_job_running(&self, job_id: i64) -> Result<(), RunnerError> {
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
    ) -> Result<(), RunnerError> {
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
    ) -> Result<(), RunnerError> {
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

    async fn get_jobs_all(&self) -> Result<Vec<Job>, RunnerError> {
        let query = r#"
            SELECT * FROM jobs
        "#;

        let jobs = sqlx::query_as::<_, Job>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(jobs)
    }

    async fn delete_jobs(&self, days: u32) -> Result<(), RunnerError> {
        let days_param = format!("-{} days", days);

        let result: Vec<Job> =
            sqlx::query_as("SELECT * FROM jobs WHERE created_at < date('now', ?)")
                .bind(days_param)
                .fetch_all(&self.pool)
                .await?;

        println!("{:#?}", result);

        Ok(())
    }
}
