use std::{borrow::Cow, fmt::Display};

use serde::Serialize;

use crate::app::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // Ensure it takes up only 1 byte
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct LogEntry<'a> {
    pub id: i64,
    pub log_timestamp: Cow<'a, str>,
    pub log_level: i64,
    pub log_target: Cow<'a, str>,
    pub log_message: Cow<'a, str>,
    pub log_line: i64,
}

impl<'a> LogEntry<'a> {
    pub fn new(
        level: LogLevel,
        message: impl Into<Cow<'a, str>>,
        target: impl Into<Cow<'a, str>>,
        line: i64,
    ) -> Self {
        let now = chrono::Utc::now().to_string();
        // // High-precision timestamp (milliseconds since epoch)
        // timestamp_ms: u64
        // let now = SystemTime::now();
        // let timestamp_ms = now
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap_or_default()
        //     .as_millis() as u64;
        //
        let level = match level {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
            LogLevel::Fatal => 5,
        };

        LogEntry {
            id: 0,
            log_timestamp: now.into(),
            log_level: level,
            log_target: target.into(),
            log_message: message.into(),
            log_line: line,
        }
    }
}

pub async fn logit(app_state: &AppState, entry: LogEntry<'_>) {
    if let Err(e) = sqlx::query_as!(
        LogEntry,
        "INSERT INTO logging (log_timestamp, log_level, log_target, log_message, log_line) VALUES ($1, $2, $3, $4, $5)",
        entry.log_timestamp,
        entry.log_level,
        entry.log_target,
        entry.log_message,
        entry.log_line,
    )
    .execute(&app_state.db)
    .await
    {
        tracing::error!("Failed to log: {}", e);
    } else {
        tracing::info!("Successful log");
    }
}
