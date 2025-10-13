use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    infra::data::crawler::crawl_kline_eastmoney,
    infra::storage::sqlite::insert_klines,
    job::model::{Job, JobError, JobResult, JobType},
};

#[async_trait]
pub trait JobHandler: Send + Sync {
    fn job_type(&self) -> JobType;
    async fn handle(&self, job: &Job) -> Result<JobResult, JobError>;
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

// ---------------------------------------------------------------
// ---------------------------------------------------------------

#[derive(Clone)]
pub struct CrawlPriceHandler {
    pub pool: Pool<Sqlite>,
}

#[derive(Serialize, Deserialize)]
pub struct CrawlPricePayload {
    pub ticker: String,
    pub url: String,
}

#[async_trait]
impl JobHandler for CrawlPriceHandler {
    fn job_type(&self) -> JobType {
        JobType::CrawlPrice
    }

    async fn handle(&self, job: &Job) -> Result<JobResult, JobError> {
        let payload: CrawlPricePayload =
            serde_json::from_value(job.payload.clone()).map_err(JobError::Serialization)?;

        let klines = match crawl_kline_eastmoney(&payload.url).await {
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
        match insert_klines(&self.pool, klines).await {
            Ok(()) => Ok(JobResult {
                success: true,
                output: Some(serde_json::json!({
                    "crawled price": format!("{}", payload.ticker),
                })),
                error: None,
            }),
            Err(e) => Err(JobError::Other(e.to_string())),
        }
    }
}

// ---------------------------------------------------------------
// ---------------------------------------------------------------

pub struct CrawlTestHandler;

#[derive(Serialize, Deserialize)]
pub struct CrawlTestPayload {
    pub url: String,
}

#[async_trait]
impl JobHandler for CrawlTestHandler {
    fn job_type(&self) -> JobType {
        JobType::CrawlTest
    }

    async fn handle(&self, job: &Job) -> Result<JobResult, JobError> {
        let payload: CrawlTestPayload =
            serde_json::from_value(job.payload.clone()).map_err(JobError::Serialization)?;

        // FIXME: refine logging for prod and test
        println!("Crawling  {:?}", payload.url);
        let res = url_get(&payload.url).await;

        // FIXME: what is the sucess: false case?
        match res {
            Ok(data) => Ok(JobResult {
                success: true,
                output: Some(data),
                error: None,
            }),
            Err(e) => Ok(JobResult {
                success: false,
                output: None,
                error: Some(e.to_string()),
            }),
        }
    }
}

async fn url_get(url: &str) -> Result<serde_json::Value, anyhow::Error> {
    match ureq::get(url).call() {
        Ok(mut resp) => {
            let text = resp.body_mut().read_to_string()?;
            let data: serde_json::Value = serde_json::from_str(&text)?;
            Ok(data)
        }
        Err(ureq::Error::StatusCode(code)) => {
            anyhow::bail!("HTTP error: {code}",)
        }
        Err(e) => {
            anyhow::bail!("Non-HTTP error: {e}",)
        }
    }
}
