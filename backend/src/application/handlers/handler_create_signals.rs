use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        handlers::JobHandler,
        model::{Job, JobError, JobResult, JobType},
    },
    domain::{model::Signal, repository::DomainRepository, service_signal::compute_kdj},
};

// ---------------------------------------------------------------
// Compute signals
// - (ticker) -> compute all signals + write db.
// ---------------------------------------------------------------
#[derive(Clone)]
pub struct ComputeSignalHandler {
    pub repo: Arc<dyn DomainRepository>,
}

#[derive(Serialize, Deserialize)]
pub struct ComputeSignalPayload {
    pub ticker: String,
}

#[async_trait]
impl JobHandler for ComputeSignalHandler {
    fn job_type(&self) -> JobType {
        JobType::ComputeSignal
    }

    async fn handle(&self, job: &Job) -> Result<JobResult, JobError> {
        let payload: ComputeSignalPayload =
            serde_json::from_value(job.payload.clone()).map_err(JobError::Serialization)?;

        let klines = self.repo.get_klines(&payload.ticker).await?;

        // compute_kdj() for all
        let kdjs = compute_kdj(klines);
        let last_kdj = kdjs.last().unwrap();
        let signal = Signal {
            ticker: payload.ticker,
            kdj_k: last_kdj.k,
            kdj_d: last_kdj.d,
        };

        self.repo.create_signals(signal).await?;

        Ok(JobResult {
            success: true,
            output: Some(serde_json::json!({"update signals table": "ok"})),
            error: None,
        })
    }
}
