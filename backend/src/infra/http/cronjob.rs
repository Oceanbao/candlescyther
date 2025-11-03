use serde_json::json;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};

use crate::{
    application::{
        handlers::{
            handler_create_ml_sector::CreateMfSectorPayload,
            handler_create_signals_sector::CreateSignalSectorPayload,
        },
        model::{Job, JobType},
    },
    infra::{
        http::AppState,
        logging::{LogEntry, LogLevel, logit},
    },
};

// The existing function rewritten for clarity in Arc ownership
pub async fn setup_cron_jobs(app_state: AppState) -> anyhow::Result<()> {
    let scheduler = JobScheduler::new().await?;

    // --- Initial Clones (Necessary for long-term ownership by each job) ---
    // The original `app_state` will now be implicitly dropped at the end of the function,
    // and its reference count is moved into job1_state and job2_state.
    let job1_state = app_state.clone();
    let job2_state = app_state.clone();

    // ----------------------
    // Job 1: Runs every weekday at 18:00 (6 PM)
    // ----------------------
    scheduler
        .add(
            JobBuilder::new()
                .with_timezone(chrono_tz::Tz::Asia__Shanghai)
                .with_cron_job_type()
                .with_schedule("0 0 18 * * 1-5")
                .unwrap()
                .with_run_async(Box::new(move |_uuid, _l| {
                    // Inner Clone: Necessary for every run, as `cron_create_mf_sector` consumes ownership.
                    let app_state_for_run = job1_state.clone();
                    Box::pin(async move {
                        if let Err(e) = cron_create_mf_sector(app_state_for_run).await {
                            tracing::error!("Cron job 1 (18:00) failed: {}", e);
                        }
                    })
                }))
                .build()
                .unwrap(),
        )
        .await?;

    // ----------------------
    // Job 2: Runs every weekday at 19:00 (7 PM)
    // ----------------------
    scheduler
        .add(
            JobBuilder::new()
                .with_timezone(chrono_tz::Tz::Asia__Shanghai)
                .with_cron_job_type()
                .with_schedule("0 0 19 * * 1-5")
                .unwrap()
                .with_run_async(Box::new(move |_uuid, _l| {
                    // Inner Clone: Necessary for every run, as `cron_create_mf_sector` consumes ownership.
                    let app_state_for_run = job2_state.clone();
                    Box::pin(async move {
                        if let Err(e) = cron_create_signals_sector(app_state_for_run).await {
                            tracing::error!("Cron job 2 (19:00) failed: {}", e);
                        }
                    })
                }))
                .build()
                .unwrap(),
        )
        .await?;

    scheduler.start().await?;

    Ok(())
}

async fn cron_create_mf_sector(state: AppState) -> anyhow::Result<()> {
    let job = Job::new(JobType::CreateMfSector, json!(CreateMfSectorPayload {}));

    if let Err(e) = state.runner.repo_job.create_jobs(vec![job]).await {
        logit(
            &state,
            LogEntry::new(
                LogLevel::Error,
                "failed to create_jobs in cron_create_mf_sector",
                "http/cronjob.rs",
                48,
            ),
        )
        .await;
        return Err(anyhow::anyhow!("{e}"));
    }

    tokio::spawn(async move {
        if let Err(e) = state.runner.run().await {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("runner error: {}", e),
                    "http/cronjob.rs",
                    63,
                ),
            )
            .await;
        }
    });

    Ok(())
}

async fn cron_create_signals_sector(state: AppState) -> anyhow::Result<()> {
    let tickers: Vec<String> = match state.runner.repo_domain.get_sector_tickers().await {
        Ok(stocks) => stocks.iter().map(|t| t.ticker.clone()).collect(),
        Err(e) => {
            return Err(anyhow::anyhow!("{e}"));
        }
    };

    let mut jobs = vec![];
    for ticker in &tickers {
        jobs.push(Job::new(
            JobType::CreateSignalSector,
            json!(CreateSignalSectorPayload {
                ticker: ticker.to_string(),
            }),
        ));
    }

    if let Err(e) = state.runner.repo_job.create_jobs(jobs).await {
        logit(
            &state,
            LogEntry::new(
                LogLevel::Error,
                "failed to create_jobs in cron_create_signals_sector",
                "http/cronjob.rs",
                106,
            ),
        )
        .await;
        return Err(anyhow::anyhow!("{e}"));
    }

    tokio::spawn(async move {
        if let Err(e) = state.runner.run().await {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("runner error: {}", e),
                    "http/cronjob.rs",
                    121,
                ),
            )
            .await;
        }
    });

    Ok(())
}
