use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::job::{
    handler::JobHandlerRegistry,
    model::{Job, JobStatus},
    repository::JobRepository,
};

pub struct JobRunner {
    repository: Arc<dyn JobRepository>,
    handler_registry: Arc<JobHandlerRegistry>,
    concurrency_limit: Arc<Semaphore>,
    wait_ms: usize,
    batch_size: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Job execution error: {message}")]
    Execution { job_id: i64, message: String },
}

impl JobRunner {
    pub fn new(
        repository: Arc<dyn JobRepository>,
        handler_registry: Arc<JobHandlerRegistry>,
        max_concurrent_jobs: usize,
        wait_ms: usize,
        batch_size: usize,
    ) -> Self {
        Self {
            repository,
            handler_registry,
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent_jobs)),
            wait_ms,
            batch_size,
        }
    }

    // NOTE: This is handled by the initiator in a server handler.
    // If err, it's non-task error and should be logged.
    pub async fn run(&self) -> Result<(), RunnerError> {
        let mut timer = tokio::time::interval(tokio::time::Duration::from_millis(
            self.wait_ms.try_into().unwrap(),
        ));

        loop {
            println!("------- run loop -------");

            timer.tick().await;

            // FIXME: refine the num jobs each loop gets
            // NOTE: this is diff the `concurrency_limit`
            let pending_jobs = self.repository.get_pending_jobs(self.batch_size).await?;

            if pending_jobs.is_empty() {
                return Ok(());
                // continue
            }

            let mut handles = Vec::new();

            for job in pending_jobs {
                let permit = self
                    .concurrency_limit
                    .clone()
                    .acquire_owned()
                    .await
                    .unwrap();
                let repository = self.repository.clone();
                let handler_registry = self.handler_registry.clone();

                let handle = tokio::spawn(async move {
                    let result = Self::process_job(job, repository, handler_registry).await;
                    drop(permit); // Release the semaphore permit
                    result
                });

                handles.push(handle);
            }

            // Wait for all jobs in this batch to complete
            for handle in handles {
                match handle.await {
                    Ok(a) => {
                        if let Err(e) = a {
                            match e {
                                RunnerError::Execution { job_id, message } => {
                                    self.repository
                                        .update_job_status(job_id, JobStatus::Error, Some(message))
                                        .await?;
                                }
                                RunnerError::Database(e) => {
                                    eprintln!("{e}");
                                }
                            }
                        }
                    }
                    Err(join_err) => {
                        eprintln!("Log this join handler error {join_err}");
                    }
                }
            }
        }
    }

    async fn process_job(
        job: Job,
        repository: Arc<dyn JobRepository>,
        handler_registry: Arc<JobHandlerRegistry>,
    ) -> Result<(), RunnerError> {
        // Mark job as running
        repository.mark_job_running(job.id).await?;

        // Find appropriate handler
        let handler =
            handler_registry
                .get_handler(&job.job_type)
                .ok_or_else(|| RunnerError::Execution {
                    job_id: job.id,
                    message: format!("No handler for job type: {:?}", job.job_type),
                })?;

        // Execute the job
        // FIXME: refine the logic in error
        match handler.handle(&job).await {
            // Job result
            Ok(result) => {
                if result.success {
                    repository.mark_job_done(job.id, result.output).await?;
                } else {
                    repository
                        .update_job_status(job.id, JobStatus::Error, result.error)
                        .await?;
                }
                Ok(())
            }
            // Job error
            Err(e) => {
                repository
                    .update_job_status(job.id, JobStatus::Error, Some(e.to_string()))
                    .await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    use crate::{
        domain::model::Kline,
        job::{
            handler::{
                CrawlPriceHandler, CrawlPricePayload, CrawlTestHandler, CrawlTestPayload,
                JobHandlerRegistry,
            },
            model::{Job, JobStatus, JobType},
            repository::SqliteJobRepository,
            runner::JobRunner,
        },
    };

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        let database_url = "sqlite::memory:";
        let pool = SqlitePoolOptions::new().connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn test_jobs_dummy_crawl() {
        let pool = setup_test_db().await.unwrap();
        let repository = Arc::new(SqliteJobRepository::new(pool));

        let crawltest_handler = CrawlTestHandler;

        let mut handler_registry = JobHandlerRegistry::new();
        handler_registry.register_handler(Arc::new(crawltest_handler));

        let concurrency = 2;
        let wait_ms = 500;
        let batch_size = concurrency;

        let runner = JobRunner::new(
            repository.clone(),
            Arc::new(handler_registry),
            concurrency,
            wait_ms,
            batch_size,
        );

        let mut jobs = vec![];
        for _ in 0..3 {
            jobs.push((
                JobType::CrawlTest,
                json!(CrawlTestPayload {
                    url: "https://dummyjson.com/http/200".to_string(),
                }),
            ));
        }
        for _ in 0..3 {
            jobs.push((
                JobType::CrawlTest,
                json!(CrawlTestPayload {
                    url: "https://dummyjson.com/http/404/bad".to_string(),
                }),
            ));
        }

        runner.repository.create_jobs(jobs).await.unwrap();

        runner.run().await.unwrap();

        let query = r#"
            SELECT * FROM jobs
        "#;

        let jobs = sqlx::query_as::<_, Job>(query)
            .fetch_all(&repository.pool)
            .await;

        assert!(jobs.is_ok());
        let jobs = jobs.unwrap();
        assert_eq!(jobs.len(), 6);

        for job in jobs {
            if job.job_status == JobStatus::Done {
                assert_eq!(job.job_type, JobType::CrawlTest);
                assert_eq!(job.job_status, JobStatus::Done);
                assert_eq!(job.error_message, None);
                assert_eq!(job.payload, json!({ "message": "OK", "status": 200 }));
            }
            if job.job_status == JobStatus::Error {
                assert_eq!(job.job_type, JobType::CrawlTest);
                assert_eq!(job.job_status, JobStatus::Error);
                // FIXME: seems error_message needs change
                assert_eq!(job.error_message, Some("HTTP error: 404".to_string()));
                assert_eq!(
                    job.payload,
                    json!({ "url": "https://dummyjson.com/http/404/bad" })
                );
            }
        }
    }

    #[tokio::test]
    async fn test_jobs_crawl_price_eastmoney() {
        let pool = setup_test_db().await.unwrap();
        let repository = Arc::new(SqliteJobRepository::new(pool));

        let crawlprice_handler = CrawlPriceHandler {
            pool: repository.pool.clone(),
        };

        let mut handler_registry = JobHandlerRegistry::new();
        handler_registry.register_handler(Arc::new(crawlprice_handler));

        let concurrency = 1;
        let wait_ms = 1000;
        let batch_size = concurrency;

        let runner = JobRunner::new(
            repository.clone(),
            Arc::new(handler_registry),
            concurrency,
            wait_ms,
            batch_size,
        );

        let tickers = ["105.APP", "105.TSLA", "1.600635", "1.688981"];
        let jobs: Vec<_> = tickers
            .into_iter()
            .map(|ticker| {
                (
                    JobType::CrawlPrice,
                    json!(CrawlPricePayload {
                        ticker: ticker.to_string(),
                        url: format!("https://54.push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35106707668456928451_1695010059469&secid={}&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=101&fqt=1&beg=20110101&end=20110202&lmt=1200&_=1695010059524", ticker)
                    }),
                )
            })
            .collect();

        runner.repository.create_jobs(jobs).await.unwrap();

        runner.run().await.unwrap();

        let query = r#"
            SELECT * FROM jobs
        "#;

        let jobs = sqlx::query_as::<_, Job>(query)
            .fetch_all(&repository.pool)
            .await;

        assert!(jobs.is_ok());
        let jobs = jobs.unwrap();
        assert_eq!(jobs.len(), tickers.len());

        for (i, job) in jobs.iter().enumerate() {
            assert_eq!(job.job_type, JobType::CrawlPrice);
            assert_eq!(job.job_status, JobStatus::Done);
            assert_eq!(job.payload, json!({ "crawled price": tickers[i] }));
            assert_eq!(job.error_message, None);
        }

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from kline")
            .fetch_one(&repository.pool)
            .await
            .unwrap();

        assert_eq!(count, 27);

        let results: Vec<Kline> = sqlx::query_as(
            r#"
            SELECT k_ticker, k_date, k_open, k_high, k_low, k_close, k_volume, k_value
            FROM kline 
            WHERE k_ticker = '105.TSLA'
            ORDER BY k_date ASC
            LIMIT 2
            "#,
        )
        .fetch_all(&repository.pool)
        .await
        .unwrap();

        assert_eq!(results[0].k_ticker, "105.TSLA");
        assert_eq!(results[1].k_ticker, "105.TSLA");
        assert_eq!(results[0].k_date, 20110126);
        assert_eq!(results[1].k_date, 20110127);
    }
}
