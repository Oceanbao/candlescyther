use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use sqlx::{Row, SqlitePool};

// --------------------------------------------------
// MODEL
// --------------------------------------------------

#[derive(Default, Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "job_status")]
#[sqlx(rename_all = "lowercase")]
pub enum JobStatus {
    #[default]
    Pending,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Hash, Eq, PartialEq)]
#[sqlx(type_name = "job_type")]
#[sqlx(rename_all = "lowercase")]
pub enum JobType {
    CrawlPrice,
    // Add more job types as needed
}

#[derive(Debug, Clone, FromRow)]
pub struct Job {
    pub id: i64,
    pub job_status: JobStatus,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

// --------------------------------------------------
// REPOSITORY
// --------------------------------------------------

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
    pool: SqlitePool,
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
        let query = r#"
            INSERT INTO jobs (job_type, payload, status, created_at, updated_at)
            VALUES ($1, $2, 'pending', NOW(), NOW())
            RETURNING id
        "#;

        let row = sqlx::query(query)
            .bind(job_type)
            .bind(payload)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("id"))
    }

    async fn get_pending_jobs(&self, limit: usize) -> Result<Vec<Job>, JobError> {
        let query = r#"
            SELECT * FROM jobs
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        "#;

        let jobs = sqlx::query_as::<_, Job>(query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        Ok(jobs)
    }

    async fn mark_job_running(&self, job_id: i64) -> Result<(), JobError> {
        let query = r#"
            UPDATE jobs
            SET status = 'running', updated_at = NOW()
            WHERE id = $1
        "#;
        sqlx::query(query).bind(job_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn mark_job_done(
        &self,
        job_id: i64,
        output: Option<serde_json::Value>,
    ) -> Result<(), JobError> {
        let query = r#"
            UPDATE jobs
            SET status = 'done', updated_at = NOW(), payload = $2
            WHERE id = $1
        "#;
        sqlx::query(query)
            .bind(job_id)
            .bind(output)
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
        let query = r#"
            UPDATE jobs
            SET status = $2, updated_at = NOW(), error_message = $3
            WHERE id = $1
        "#;
        sqlx::query(query)
            .bind(job_id)
            .bind(status)
            .bind(error_message)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait JobHandler: Send + Sync {
    fn job_type(&self) -> JobType;
    async fn handle(&self, job: &Job) -> Result<JobResult, String>;
}

// Type alias for boxed handlers
pub type BoxedJobHandler = Arc<dyn JobHandler>;

#[derive(Default)]
pub struct JobHandlerRegistry {
    handlers: HashMap<JobType, BoxedJobHandler>,
}

impl JobHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, handler: BoxedJobHandler) {
        let job_type = handler.job_type();
        self.handlers.insert(job_type, handler);
    }

    pub fn get_handler(&self, job_type: &JobType) -> Option<&BoxedJobHandler> {
        self.handlers.get(job_type)
    }
}

pub struct CrawlPriceHandler;

#[async_trait]
impl JobHandler for CrawlPriceHandler {
    fn job_type(&self) -> JobType {
        JobType::CrawlPrice
    }

    async fn handle(&self, job: &Job) -> Result<JobResult, String> {
        let payload: CrawlPricePayload = serde_json::from_value(job.payload.clone())
            .map_err(|e| format!("Failed to parse processing payload: {}", e))?;

        // Simulate data processing
        println!("Crawling  {:?}", payload.ticker);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(JobResult {
            success: true,
            output: Some(serde_json::json!({
                "crawled": format!("{}", payload.ticker),
            })),
            error: None,
        })
    }
}

#[derive(Deserialize)]
struct CrawlPricePayload {
    ticker: String,
}

use tokio::sync::Semaphore;

pub struct JobRunner {
    repository: Arc<dyn JobRepository>,
    handler_registry: Arc<JobHandlerRegistry>,
    concurrency_limit: Arc<Semaphore>,
}

impl JobRunner {
    pub fn new(
        repository: Arc<dyn JobRepository>,
        handler_registry: Arc<JobHandlerRegistry>,
        max_concurrent_jobs: usize,
    ) -> Self {
        Self {
            repository,
            handler_registry,
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent_jobs)),
        }
    }

    pub async fn run(&self) -> Result<(), JobError> {
        let mut looped = 0;
        loop {
            let pending_jobs = self.repository.get_pending_jobs(10).await?;

            if pending_jobs.is_empty() {
                // Wait before checking again
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }

            let mut handles = Vec::new();

            for job in pending_jobs {
                let permit = self
                    .concurrency_limit
                    .clone()
                    .acquire_owned()
                    .await
                    .unwrap();
                let repository = self.repository.clone();
                let handler_registry = self.handler_registry.clone();

                let handle = tokio::spawn(async move {
                    let result = Self::process_job(job, repository, handler_registry).await;
                    drop(permit); // Release the semaphore permit
                    result
                });

                handles.push(handle);
            }

            // Wait for all jobs in this batch to complete
            for handle in handles {
                if let Err(e) = handle.await {
                    eprintln!("Job processing task panicked: {:?}", e);
                }
            }

            looped += 1;
            if looped == 2 {
                return Ok(());
            }
        }
    }

    async fn process_job(
        job: Job,
        repository: Arc<dyn JobRepository>,
        handler_registry: Arc<JobHandlerRegistry>,
    ) -> Result<(), JobError> {
        // Mark job as running
        repository.mark_job_running(job.id).await?;

        // Find appropriate handler
        let handler = handler_registry.get_handler(&job.job_type).ok_or_else(|| {
            JobError::Execution(format!("No handler for job type: {:?}", job.job_type))
        })?;

        // Execute the job
        match handler.handle(&job).await {
            Ok(result) => {
                if result.success {
                    repository.mark_job_done(job.id, result.output).await?;
                } else {
                    repository
                        .update_job_status(job.id, JobStatus::Error, result.error)
                        .await?;
                }
                Ok(())
            }
            Err(error_msg) => {
                repository
                    .update_job_status(job.id, JobStatus::Error, Some(error_msg))
                    .await?;
                Err(JobError::Execution("Job handler failed".to_string()))
            }
        }
    }
}
