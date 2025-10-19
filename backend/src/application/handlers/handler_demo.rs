use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::application::{
    handlers::JobHandler,
    model::{Job, JobError, JobResult, JobType},
};

// ---------------------------------------------------------------
// Test Crawl
// NOTE: delete later
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
