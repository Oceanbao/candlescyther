use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        handlers::JobHandler,
        model::{Job, JobError, JobResult, JobType},
    },
    domain::{
        model::Signal,
        repository::DomainRepository,
        service_signal::{compute_boll_dist, compute_kdj},
    },
    infra::data::kline::{UrlKlineEastmoney, crawl_kline_eastmoney},
};

// ---------------------------------------------------------------
// Create Signals Sector
// - crawl all klines 1D for one sector (get from stocks table)
// - compute signals from klines
// - save signals to db
// ---------------------------------------------------------------
#[derive(Clone)]
pub struct CreateSignalSectorHandler {
    pub repo: Arc<dyn DomainRepository>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSignalSectorPayload {
    pub ticker: String,
}

#[async_trait]
impl JobHandler for CreateSignalSectorHandler {
    fn job_type(&self) -> JobType {
        JobType::CreateSignalSector
    }

    async fn handle(&self, job: &Job) -> Result<JobResult, JobError> {
        let payload: CreateSignalSectorPayload =
            serde_json::from_value(job.payload.clone()).map_err(JobError::Serialization)?;

        let url = UrlKlineEastmoney::new(&payload.ticker, "0", "20500101", false);

        let klines = crawl_kline_eastmoney(url).await?;

        let kdjs = compute_kdj(&klines);
        let last_kdj = kdjs.last().unwrap();
        let boll_dist = compute_boll_dist(&klines);
        let signal = Signal {
            ticker: payload.ticker.clone(),
            kdj_k: last_kdj.k,
            kdj_d: last_kdj.d,
            boll_dist,
        };

        self.repo.create_signals_sector(signal).await?;

        Ok(JobResult {
            success: true,
            output: Some(serde_json::json!({"inserted signals_sector table": &payload.ticker})),
            error: None,
        })
    }
}
