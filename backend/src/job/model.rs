use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Default, Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
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

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Job execution error: {0}")]
    Other(String),
}
