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
    infra::data::{
        kline::{UrlKlineEastmoney, crawl_kline_eastmoney},
        stock::{UrlStockEastmoney, crawl_stock_eastmoney},
    },
};

// ---------------------------------------------------------------
// Create Stock
// NOTE: extra flag on DELETE existing records for update purpose.
// - (ticker) -> crawl meta and save
// - (ticker) -> crawl klines and save
// - (ticker) -> compute signals and save
// ---------------------------------------------------------------

#[derive(Clone)]
pub struct CreateStockHandler {
    pub repo: Arc<dyn DomainRepository>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateStockPayload {
    pub ticker: String,
    pub update: bool,
}

#[async_trait]
impl JobHandler for CreateStockHandler {
    fn job_type(&self) -> JobType {
        JobType::CreateStock
    }

    // FIX: later abstract data sourcing into port.
    async fn handle(&self, job: &Job) -> Result<JobResult, JobError> {
        let payload: CreateStockPayload =
            serde_json::from_value(job.payload.clone()).map_err(JobError::Serialization)?;

        let mut check_skip = true;

        // NOTE: If update, delete all records per ticker.
        if payload.update {
            // NOTE: This delete all records related to the ticker.
            self.repo.delete_stocks(&[&payload.ticker]).await?;
            check_skip = false;
        }

        // NOTE: Else, skip the job if ticker exists in stocks table.
        // `ticker` is indexed so it's fast lookup. e.g. 105.TSLA
        // (SELECT 1 FROM table_name WHERE column_name = ? LIMIT 1;)
        if check_skip
            && self
                .repo
                .get_stock(&payload.ticker)
                .await
                .is_ok_and(|s| s.ticker == payload.ticker)
        {
            return Ok(JobResult {
                success: true,
                output: Some(serde_json::json!({
                    "job skipped for ticker already exists": format!("{}", payload.ticker),
                })),
                error: None,
            });
        }

        // Step 1: crawl stock meta.
        // FIX: put url into data/ as a Entity with new or sth
        // allow more config on crawling
        let stock = match crawl_stock_eastmoney(UrlStockEastmoney::new(&payload.ticker)).await {
            Ok(stock) => stock,
            Err(e) => {
                return Ok(JobResult {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                });
            }
        };

        self.repo.create_stock(stock).await?;

        // Step 2: crawl klines of the stock.
        let klines =
            match crawl_kline_eastmoney(UrlKlineEastmoney::new(&payload.ticker, "0", "20500101"))
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

        self.repo.create_klines(&payload.ticker, &klines).await?;

        // Step 3: compute signals from klines.
        let kdjs = compute_kdj(&klines);
        let last_kdj = kdjs.last().unwrap();
        let boll_dist = compute_boll_dist(&klines);
        let signal = Signal {
            ticker: payload.ticker.clone(),
            kdj_k: last_kdj.k,
            kdj_d: last_kdj.d,
            boll_dist,
        };

        self.repo.create_signals(signal).await?;

        Ok(JobResult {
            success: true,
            output: Some(serde_json::json!({
                "created stock": format!("{}", payload.ticker),
            })),
            error: None,
        })
    }
}
