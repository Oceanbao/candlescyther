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
    ) -> Self {
        Self {
            repository,
            handler_registry,
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent_jobs)),
        }
    }

    // NOTE: This is handled by the initiator in a server handler.
    // If err, it's non-task error and should be logged.
    pub async fn run(&self) -> Result<(), RunnerError> {
        let wait_sec = 1;
        let mut timer = tokio::time::interval(tokio::time::Duration::from_secs(wait_sec));
        let mut looped = 0;

        loop {
            println!("run loop {looped}");

            timer.tick().await;

            // FIXME: refine the num jobs each loop gets
            // NOTE: this is diff the `concurrency_limit`
            let pending_jobs = self.repository.get_pending_jobs(10).await?;

            if pending_jobs.is_empty() {
                // FIXME: refine this
                looped += 1;
                if looped == 2 {
                    return Ok(());
                }

                continue;
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

            looped += 1;
            if looped == 2 {
                return Ok(());
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

    use crate::job::{
        handler::{CrawlPriceHandler, CrawlPricePayload, JobHandlerRegistry},
        model::{Job, JobStatus, JobType},
        repository::SqliteJobRepository,
        runner::JobRunner,
    };

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        let database_url = "sqlite::memory:";
        let pool = SqlitePoolOptions::new().connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn test_jobs() {
        let pool = setup_test_db().await.unwrap();
        let repository = Arc::new(SqliteJobRepository::new(pool));

        let crawlprice_handler = CrawlPriceHandler;

        let mut handler_registry = JobHandlerRegistry::new();
        handler_registry.register_handler(Arc::new(crawlprice_handler));

        let concurrency = 5;

        let runner = JobRunner::new(repository.clone(), Arc::new(handler_registry), concurrency);

        let mut jobs = vec![];
        for i in 0..15 {
            jobs.push((
                JobType::CrawlPrice,
                json!(CrawlPricePayload {
                    ticker: format!("10{}.APPL", i + 1),
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
        assert_eq!(jobs.len(), 15);

        let first = jobs.first().unwrap();
        assert_eq!(first.job_type, JobType::CrawlPrice);
        assert_eq!(first.job_status, JobStatus::Done);
        assert_eq!(first.payload, json!({ "crawled": "101.APPL"}));

        let last = jobs.last().unwrap();
        assert_eq!(last.job_type, JobType::CrawlPrice);
        assert_eq!(last.job_status, JobStatus::Done);
    }
}
