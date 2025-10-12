use serde::Serialize;
use sqlx::{SqlitePool, prelude::FromRow};

// HealthCheck record for serialization
#[derive(Serialize, Debug, FromRow)]
pub struct User {
    id: i64,
    user_name: String,
    user_role: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}
