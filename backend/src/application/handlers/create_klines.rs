use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        handlers::JobHandler,
        model::{Job, JobError, JobResult, JobType},
    },
    domain::repository::DomainRepository,
    infra::data::kline::{UrlKlineEastmoney, crawl_kline_eastmoney},
};

// ---------------------------------------------------------------
// Crawl Price
// - (ticker, url) -> [Kline] and write db
// ---------------------------------------------------------------
#[derive(Clone)]
pub struct CreateKlineHandler {
    pub repo: Arc<dyn DomainRepository>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateKlinePayload {
    pub ticker: String,
    pub start: String,
    pub end: String,
}

#[async_trait]
impl JobHandler for CreateKlineHandler {
    fn job_type(&self) -> JobType {
        JobType::CreateKline
    }

    async fn handle(&self, job: &Job) -> Result<JobResult, JobError> {
        let payload: CreateKlinePayload =
            serde_json::from_value(job.payload.clone()).map_err(JobError::Serialization)?;

        // FIX: make port of data sourcing. i.e. data.crawl_kline()
        let klines = match crawl_kline_eastmoney(UrlKlineEastmoney::new(
            &payload.ticker,
            &payload.start,
            &payload.end,
            true,
        ))
        .await
        {
            Ok(klines) => klines,
            Err(e) => {
                return Ok(JobResult {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                });
            }
        };

        // FIXME: what is the sucess: false case?
        match self.repo.create_klines(&payload.ticker, &klines).await {
            Ok(()) => Ok(JobResult {
                success: true,
                output: Some(serde_json::json!({
                    "crawled price": format!("{}", payload.ticker),
                })),
                error: None,
            }),
            Err(e) => Err(JobError::Unknown(e)),
        }
    }
}
