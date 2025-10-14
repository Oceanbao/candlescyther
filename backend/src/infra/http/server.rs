use serde::Serialize;
use sqlx::{SqlitePool, prelude::FromRow};

use crate::job::runner::JobRunner;

// HealthCheck record for serialization
#[derive(Serialize, Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub user_name: String,
    pub user_role: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub runner: JobRunner,
}
