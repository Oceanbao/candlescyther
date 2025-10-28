use std::sync::Arc;
use tracing::debug;

use tokio::sync::Semaphore;

use crate::{
    application::{
        handlers::JobHandlerRegistry,
        model::{Job, JobStatus},
        repository::JobRepository,
    },
    domain::repository::DomainRepository,
};

/// Main engine of the application that drives query/command.
/// Its states include repositories of all domains, registry, config.
#[derive(Clone)]
pub struct JobRunner {
    pub repo_domain: Arc<dyn DomainRepository>,
    pub repo_job: Arc<dyn JobRepository>,
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
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl JobRunner {
    pub fn new(
        repo_domain: Arc<dyn DomainRepository>,
        repo_job: Arc<dyn JobRepository>,
        handler_registry: Arc<JobHandlerRegistry>,
        max_concurrent_jobs: usize,
        wait_ms: usize,
        batch_size: usize,
    ) -> Self {
        Self {
            repo_domain,
            repo_job,
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
            debug!("------- run loop -------");

            timer.tick().await;

            // FIXME: refine the num jobs each loop gets
            // NOTE: this != `concurrency_limit`
            let pending_jobs = self.repo_job.get_pending_jobs(self.batch_size).await?;

            // FIX: possible refine
            if pending_jobs.is_empty() {
                return Ok(());
                // continue
            }

            let mut handles = Vec::new();

            for job in pending_jobs {
                debug!("== job: {:#?} ==", job.job_type);

                let permit = self
                    .concurrency_limit
                    .clone()
                    .acquire_owned()
                    .await
                    .unwrap();

                let runner = self.clone();

                let handle = tokio::spawn(async move {
                    let result = runner.process_job(job).await;
                    drop(permit); // Release the semaphore permit
                    result
                });

                handles.push(handle);
            }

            // FIX: refine error management
            // Wait for all jobs in this batch to complete
            for handle in handles {
                match handle.await {
                    Ok(a) => {
                        if let Err(e) = a {
                            match e {
                                RunnerError::Execution { job_id, message } => {
                                    self.repo_job
                                        .update_job_status(job_id, JobStatus::Error, Some(message))
                                        .await?;
                                }
                                RunnerError::Database(e) => {
                                    tracing::error!("{e}");
                                }
                                RunnerError::Unknown(e) => {
                                    tracing::error!("{e}");
                                }
                            }
                        }
                    }
                    Err(join_err) => {
                        tracing::error!("Log this join handler error {join_err}");
                    }
                }
            }
        }
    }

    async fn process_job(&self, job: Job) -> Result<(), RunnerError> {
        // Mark job as running
        self.repo_job.mark_job_running(job.id).await?;

        // Find appropriate handler
        let handler = self
            .handler_registry
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
                    self.repo_job.mark_job_done(job.id, result.output).await?;
                } else {
                    self.repo_job
                        .update_job_status(job.id, JobStatus::Error, result.error)
                        .await?;
                }
                Ok(())
            }
            // Job error
            Err(e) => {
                self.repo_job
                    .update_job_status(job.id, JobStatus::Error, Some(e.to_string()))
                    .await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, vec};

    use serde_json::json;
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    use crate::{
        application::{
            handlers::{
                JobHandlerRegistry,
                handler_create_klines::{CrawlPriceHandler, CrawlPricePayload},
                handler_create_signals::{ComputeSignalHandler, ComputeSignalPayload},
                handler_create_stock::{CreateStockHandler, CreateStockPayload},
            },
            model::{Job, JobStatus, JobType},
            runner::JobRunner,
        },
        infra::storage::{
            repo_domain_sqlite::SqliteDomainRepository, repo_job_sqlite::SqliteJobRepository,
        },
    };

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        let database_url = "sqlite::memory:";
        let pool = SqlitePoolOptions::new().connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }

    async fn setup_runner(pool: SqlitePool) -> Result<JobRunner, anyhow::Error> {
        let repo_domain = Arc::new(SqliteDomainRepository::new(pool.clone()));
        let repo_job = Arc::new(SqliteJobRepository::new(pool.clone()));

        let crawlprice_handler = CrawlPriceHandler {
            repo: repo_domain.clone(),
        };
        let compute_signal_handler = ComputeSignalHandler {
            repo: repo_domain.clone(),
        };
        let create_stock_handler = CreateStockHandler {
            repo: repo_domain.clone(),
        };

        let mut handler_registry = JobHandlerRegistry::new();
        handler_registry.register_handlers(vec![
            Arc::new(crawlprice_handler),
            Arc::new(compute_signal_handler),
            Arc::new(create_stock_handler),
        ]);

        let concurrency = 2;
        let wait_ms = 3000;
        let batch_size = concurrency;

        Ok(JobRunner::new(
            repo_domain,
            repo_job,
            Arc::new(handler_registry),
            concurrency,
            wait_ms,
            batch_size,
        ))
    }

    #[tokio::test]
    #[ignore = "network call to eastmoney"]
    async fn test_jobs_compute_signal_with_crawl() {
        let pool = setup_test_db().await.unwrap();
        let runner = setup_runner(pool.clone()).await.unwrap();

        let tickers = ["105.APP", "105.TSLA", "1.600635", "1.688981"];
        let jobs: Vec<_> = tickers
            .into_iter()
            .map(|ticker| {
                Job::new(
                    JobType::CrawlPrice,
                    json!(CrawlPricePayload {
                        ticker: ticker.to_string(),
                        start: "0".to_string(),
                        end: "20500101".to_string(),
                    }),
                )
            })
            .collect();

        runner.repo_job.create_jobs(jobs).await.unwrap();
        runner.run().await.unwrap();

        // Compute Signal Job
        let jobs: Vec<_> = tickers
            .into_iter()
            .map(|ticker| {
                Job::new(
                    JobType::ComputeSignal,
                    json!(ComputeSignalPayload {
                        ticker: ticker.to_string(),
                    }),
                )
            })
            .collect();
        runner.repo_job.create_jobs(jobs).await.unwrap();
        runner.run().await.unwrap();

        let query = r#"
            SELECT * FROM jobs WHERE job_type = ?
        "#;

        let jobs = sqlx::query_as::<_, Job>(query)
            .bind("computesignal")
            .fetch_all(&pool)
            .await;

        assert!(jobs.is_ok());
        let jobs = jobs.unwrap();
        assert_eq!(jobs.len(), 4);

        for job in jobs {
            assert_eq!(job.job_type, JobType::ComputeSignal);
            assert_eq!(job.job_status, JobStatus::Done);
            assert_eq!(job.payload, json!({ "update signals table": "ok" }));
            assert_eq!(job.error_message, None);
        }

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from signals")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from signals_us")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    #[ignore = "network call to eastmoney"]
    async fn test_create_stock() {
        let pool = setup_test_db().await.unwrap();
        let runner = setup_runner(pool.clone()).await.unwrap();

        let tickers = ["105.APP", "105.TSLA", "1.600635", "1.688981"];
        let jobs: Vec<_> = tickers
            .into_iter()
            .map(|ticker| {
                Job::new(
                    JobType::CreateStock,
                    json!(CreateStockPayload {
                        ticker: ticker.to_string(),
                        update: false,
                    }),
                )
            })
            .collect();

        runner.repo_job.create_jobs(jobs).await.unwrap();
        runner.run().await.unwrap();

        let signals = runner.repo_domain.get_signals_all().await.unwrap();
        assert_eq!(signals.len(), 2);

        let signals_us = runner.repo_domain.get_signals_all_us().await.unwrap();
        assert_eq!(signals_us.len(), 2);

        let stocks = runner.repo_domain.get_stock_all().await.unwrap();
        assert_eq!(stocks.len(), tickers.len());

        let klines = runner.repo_domain.get_klines("105.APP").await.unwrap();
        assert!(klines.len() as i64 > 230);

        let klines = runner.repo_domain.get_klines("1.600635").await.unwrap();
        assert!(klines.len() as i64 > 230);
    }
}
