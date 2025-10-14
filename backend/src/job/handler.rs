use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    domain::{
        model::{Kline, Signal},
        signal::compute_kdj,
    },
    infra::{data::crawler::crawl_kline_eastmoney, storage::sqlite::insert_klines},
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

// ---------------------------------------------------------------
// ---------------------------------------------------------------

#[derive(Clone)]
pub struct ComputeSignalHandler {
    pub pool: Pool<Sqlite>,
}

#[derive(Serialize, Deserialize)]
pub struct ComputeSignalPayload {}

#[async_trait]
impl JobHandler for ComputeSignalHandler {
    fn job_type(&self) -> JobType {
        JobType::ComputeSignal
    }

    async fn handle(&self, _job: &Job) -> Result<JobResult, JobError> {
        // Get distinct tickers.
        struct Ticker {
            k_ticker: String,
        }

        let tickers = sqlx::query_as!(Ticker, "SELECT DISTINCT k_ticker FROM klines")
            .fetch_all(&self.pool)
            .await?;

        let mut signals: Vec<Signal> = vec![];

        // compute_kdj() for all
        for ticker in tickers {
            let klines = sqlx::query_as!(
                Kline,
                "SELECT * FROM klines WHERE k_ticker = ? ORDER BY k_date ASC",
                ticker.k_ticker
            )
            .fetch_all(&self.pool)
            .await?;

            let kdjs = compute_kdj(klines);
            let last_kdj = kdjs.last().unwrap();
            signals.push(Signal {
                ticker: ticker.k_ticker,
                kdj_k: last_kdj.k,
                kdj_d: last_kdj.d,
            });
        }

        // delete rows
        let _ = sqlx::query!("DELETE FROM signals")
            .execute(&self.pool)
            .await?;

        // write to table
        let mut tx = self.pool.begin().await?;
        for signal in signals {
            sqlx::query!(
                "INSERT INTO signals (ticker, kdj_k, kdj_d) VALUES (?, ?, ?)",
                signal.ticker,
                signal.kdj_k,
                signal.kdj_d,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        Ok(JobResult {
            success: true,
            output: Some(serde_json::json!({"update signals table": "ok"})),
            error: None,
        })
    }
}
