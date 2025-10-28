use async_trait::async_trait;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    application::{
        handlers::JobHandler,
        model::{Job, JobError, JobResult, JobType},
    },
    domain::repository::DomainRepository,
    infra::data::moneyflow::{UrlMoneyflowSectorEastmoney, crawl_moneyflow_sector_eastmoney},
};

// ---------------------------------------------------------------
// Create Moneyflow Sector
// NOTE: decide on days to keep record.
// - () -> crawl data and save
// ---------------------------------------------------------------

#[derive(Clone)]
pub struct CreateMlSectorHandler {
    pub repo: Arc<dyn DomainRepository>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateMlSectorPayload {}

#[async_trait]
impl JobHandler for CreateMlSectorHandler {
    fn job_type(&self) -> JobType {
        JobType::CreateMlSector
    }

    // FIX: later abstract data sourcing into port.
    async fn handle(&self, _: &Job) -> Result<JobResult, JobError> {
        let url = UrlMoneyflowSectorEastmoney::default();
        let ml_records = match crawl_moneyflow_sector_eastmoney(url).await {
            Ok(res) => res,
            Err(e) => {
                return Ok(JobResult {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                });
            }
        };

        self.repo.create_ml_sector(&ml_records).await?;

        Ok(JobResult {
            success: true,
            output: Some(serde_json::json!({
                "created moneyflow sector": true,
            })),
            error: None,
        })
    }
}
