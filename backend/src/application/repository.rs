use async_trait::async_trait;

use crate::application::{
    model::{Job, JobStatus},
    runner::RunnerError,
};

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_jobs(&self, jobs: Vec<Job>) -> Result<(), RunnerError>;
    async fn get_pending_jobs(&self, limit: usize) -> Result<Vec<Job>, RunnerError>;
    async fn update_job_status(
        &self,
        job_id: i64,
        status: JobStatus,
        error_message: Option<String>,
    ) -> Result<(), RunnerError>;
    async fn mark_job_running(&self, job_id: i64) -> Result<(), RunnerError>;
    async fn mark_job_done(
        &self,
        job_id: i64,
        output: Option<serde_json::Value>,
    ) -> Result<(), RunnerError>;
    async fn get_jobs_all(&self) -> Result<Vec<Job>, RunnerError>;
}
