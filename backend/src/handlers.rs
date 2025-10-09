use axum::{Json, extract::State};
use serde_json::{Value, json};
use sqlx::sqlite::{self};

use crate::{
    app::{AppState, User},
    logging::{LogEntry, LogLevel, logit},
};

// Handler for the /check endpoint
pub async fn check_handler(State(state): State<AppState>) -> Json<Value> {
    logit(
        &state,
        LogEntry::new(LogLevel::Debug, "pre-select", "check_handler", 64),
    )
    .await;

    match sqlx::query_as::<sqlite::Sqlite, User>("SELECT * FROM user")
        .fetch_all(&state.db)
        .await
    {
        Ok(users) => Json(json!({
            "status": "ok",
            "database": "connected",
            "data": users,
        })),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(json!({
                "status": "error",
                "database": "disconnected",
                "error": e.to_string()
            }))
        }
    }
}

// Handler for /logs endpoint.
pub async fn log_handler(State(state): State<AppState>) -> Json<Value> {
    match sqlx::query_as!(LogEntry, "SELECT * FROM logging LIMIT 10")
        .fetch_all(&state.db)
        .await
    {
        Ok(logs) => Json(json!({
            "status": "ok",
            "database": "connected",
            "data": logs,
        })),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(json!({
                "status": "error",
                "database": "disconnected",
                "error": e.to_string()
            }))
        }
    }
}
