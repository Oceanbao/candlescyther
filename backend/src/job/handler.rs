use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::job::model::{Job, JobError, JobResult, JobType};

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

pub struct CrawlPriceHandler;

#[derive(Serialize, Deserialize)]
pub struct CrawlPricePayload {
    pub ticker: String,
}

#[async_trait]
impl JobHandler for CrawlPriceHandler {
    fn job_type(&self) -> JobType {
        JobType::CrawlPrice
    }

    async fn handle(&self, job: &Job) -> Result<JobResult, JobError> {
        let payload: CrawlPricePayload =
            serde_json::from_value(job.payload.clone()).map_err(JobError::Serialization)?;

        // FIXME: refine logging for prod and test
        println!("Crawling  {:?}", payload.ticker);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // FIXME: what is the sucess: false case?
        Ok(JobResult {
            success: true,
            output: Some(serde_json::json!({
                "crawled": format!("{}", payload.ticker),
            })),
            error: None,
        })
    }
}
