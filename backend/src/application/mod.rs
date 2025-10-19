use std::sync::Arc;

use crate::{
    application::{
        handlers::{
            JobHandlerRegistry, handler_create_klines::CrawlPriceHandler,
            handler_create_signals::ComputeSignalHandler, handler_create_stock::CreateStockHandler,
        },
        runner::JobRunner,
    },
    infra::storage::{
        Database, repo_domain_sqlite::SqliteDomainRepository, repo_job_sqlite::SqliteJobRepository,
    },
};

pub mod handlers;
pub mod model;
pub mod repository;
pub mod runner;

pub fn init_runner(db: &Database) -> JobRunner {
    let repo_domain = Arc::new(SqliteDomainRepository::new(db.pool.clone()));
    let repo_job = Arc::new(SqliteJobRepository::new(db.pool.clone()));

    let crawl_price_handler = CrawlPriceHandler {
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
        Arc::new(crawl_price_handler),
        Arc::new(compute_signal_handler),
        Arc::new(create_stock_handler),
    ]);

    let concurrency = 3;
    let wait_ms = 1000;
    let batch_size = concurrency;

    JobRunner::new(
        repo_domain,
        repo_job,
        Arc::new(handler_registry),
        concurrency,
        wait_ms,
        batch_size,
    )
}
