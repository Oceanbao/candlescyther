use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::ToSchema;

#[derive(Debug, Clone, FromRow, ToSchema, Serialize)]
pub struct Job {
    pub id: i64,
    pub job_status: JobStatus,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
    pub error_message: Option<String>,
}

impl Job {
    pub fn new(job_type: JobType, payload: serde_json::Value) -> Self {
        let now = chrono::Utc::now().to_string();
        // NOTE: id: 0 is fine for db auto-creates it and later
        // accessed correctly.
        Job {
            id: 0,
            job_status: JobStatus::Pending,
            job_type,
            payload,
            created_at: now.clone(),
            updated_at: now,
            error_message: None,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "job_status")]
#[sqlx(rename_all = "lowercase")]
pub enum JobStatus {
    #[default]
    Pending,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Hash, Eq, PartialEq, ToSchema)]
#[sqlx(type_name = "job_type")]
#[sqlx(rename_all = "lowercase")]
pub enum JobType {
    CreateStock,
    DeleteStock,
    CrawlPrice,
    ComputeSignal,
    CreateMfSector,
    CreateSignalSector,
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
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    #[error("Job execution error: {0}")]
    Other(String),
}
