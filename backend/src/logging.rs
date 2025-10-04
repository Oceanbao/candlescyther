use serde::Serialize;

use crate::app::AppState;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct ServerLog {
    pub id: i64,
    pub log_timestamp: String,
    pub log_level: String,
    pub log_target: String,
}

pub async fn save_log(app_state: &AppState) {
    let now = chrono::Utc::now().to_string();
    if let Err(e) = sqlx::query_as!(
        ServerLog,
        "INSERT INTO logging (log_timestamp, log_level, log_target) VALUES ($1, $2, $3)",
        now,
        "INFO",
        "target",
    )
    .fetch_one(&app_state.db)
    .await
    {
        tracing::error!("Failed to log: {}", e);
    } else {
        tracing::info!("Successful log");
    }
}
