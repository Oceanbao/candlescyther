use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::application::model::{Job, JobError, JobResult, JobType};

// FIX: refine what is needed to create a JobHandler. Unique use case/Command.
// create_klines and create_signals are UoW in create_stock.
pub mod handler_create_klines;
pub mod handler_create_signals;
pub mod handler_create_stock;

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

    pub fn register_handlers(&mut self, handlers: Vec<BoxedJobHandler>) {
        for handler in handlers {
            let job_type = handler.job_type();
            self.handlers.insert(job_type, handler);
        }
    }

    pub fn get_handler(&self, job_type: &JobType) -> Option<&BoxedJobHandler> {
        self.handlers.get(job_type)
    }
}
