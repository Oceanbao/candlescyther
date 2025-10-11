use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::job::{
    handler::JobHandlerRegistry,
    model::{Job, JobStatus},
    repository::{JobError, JobRepository},
};

pub struct JobRunner {
    repository: Arc<dyn JobRepository>,
    handler_registry: Arc<JobHandlerRegistry>,
    concurrency_limit: Arc<Semaphore>,
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

    pub async fn run(&self) -> Result<(), JobError> {
        let mut looped = 0;
        loop {
            let pending_jobs = self.repository.get_pending_jobs(10).await?;

            if pending_jobs.is_empty() {
                // Wait before checking again
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

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
                if let Err(e) = handle.await {
                    // FIXME: handle error, write error backinto db
                    eprintln!("Job processing task panicked: {:?}", e);
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
    ) -> Result<(), JobError> {
        // Mark job as running
        repository.mark_job_running(job.id).await?;

        // Find appropriate handler
        let handler = handler_registry.get_handler(&job.job_type).ok_or_else(|| {
            JobError::Execution(format!("No handler for job type: {:?}", job.job_type))
        })?;

        // Execute the job
        match handler.handle(&job).await {
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
            Err(error_msg) => {
                repository
                    .update_job_status(job.id, JobStatus::Error, Some(error_msg))
                    .await?;
                Err(JobError::Execution("Job handler failed".to_string()))
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
    async fn test_() {
        let pool = setup_test_db().await.unwrap();

        let repository = Arc::new(SqliteJobRepository::new(pool));

        // Initialize job handlers
        let mut handler_registry = JobHandlerRegistry::new();

        let crawlprice_handler = CrawlPriceHandler;

        handler_registry.register_handler(Arc::new(crawlprice_handler));

        let handler_registry = Arc::new(handler_registry);

        // Create job runner
        let runner = JobRunner::new(repository.clone(), handler_registry, 5); // Max 5 concurrent jobs

        runner
            .repository
            .create_job(
                JobType::CrawlPrice,
                json!(CrawlPricePayload {
                    ticker: "105.APPL".to_string(),
                }),
            )
            .await
            .unwrap();

        runner.run().await.unwrap();

        let query = r#"
            SELECT * FROM jobs
        "#;

        let jobs = sqlx::query_as::<_, Job>(query)
            .fetch_all(&repository.pool)
            .await;

        assert!(jobs.is_ok());

        let jobs = jobs.unwrap();
        let first = jobs.first().unwrap();

        assert_eq!(first.job_type, JobType::CrawlPrice);
        assert_eq!(first.job_status, JobStatus::Done);
        assert_eq!(first.payload, json!({ "crawled": "105.APPL"}));
    }
}
