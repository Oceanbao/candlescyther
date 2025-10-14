use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use serde::{Deserialize, Deserializer};
use serde_json::{Value, json};
use sqlx::sqlite;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::model::{Kline, Signal},
    infra::{
        http::server::{AppState, User},
        logging::{LogEntry, LogLevel, logit},
    },
    job::{
        handler::{ComputeSignalPayload, CrawlPricePayload},
        model::{Job, JobType},
    },
};

pub fn create_routes(app_state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        // /logs
        .routes(routes!(list_logs))
        // /jobs
        .routes(routes!(list_jobs))
        // /signals
        .routes(routes!(list_signals))
        // /crawl
        .routes(routes!(crawl_klines))
        // /run/signals
        .routes(routes!(compute_signals))
        // /klines
        .routes(routes!(list_klines))
        .with_state(app_state)
}

pub async fn check_handler(State(state): State<AppState>) -> Json<Value> {
    match sqlx::query_as!(User, "SELECT * FROM users")
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

/// List all jobs.
///
/// Returns all jobs.
#[utoipa::path(
    get,
    path = "/jobs",
    tag = "candlescyther",
    responses(
        (status = 200, description = "List all jobs from jobs table.", body = [Job]),
        (status = 500, description = "Database error", body = serde_json::Value)
    )
)]
pub async fn list_jobs(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as::<sqlite::Sqlite, Job>("SELECT * FROM jobs")
        .fetch_all(&state.db)
        .await
    {
        Ok(jobs) => (StatusCode::OK, Json(jobs)).into_response(),
        Err(e) => {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("failed to query database list_jobs {}", e),
                    "http/handlers.rs",
                    80,
                ),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Database error"})),
            )
                .into_response()
        }
    }
}

/// List all logs
///
/// Default 100 records.
#[utoipa::path(
    get,
    path = "/logs",
    tag = "candlescyther",
    responses(
        (status = 200, description = "List all logs from logs table.", body = [LogEntry]),
        (status = 500, description = "Database error", body = serde_json::Value)
    )
)]
pub async fn list_logs(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as!(LogEntry, "SELECT * FROM logs LIMIT 100")
        .fetch_all(&state.db)
        .await
    {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("failed to query database list_logs {}", e),
                    "http/handlers.rs",
                    118,
                ),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Database error"})),
            )
                .into_response()
        }
    }
}

/// List all signals.
///
/// Returns all signals.
#[utoipa::path(
    get,
    path = "/signals",
    tag = "candlescyther",
    responses(
        (status = 200, description = "List all signals from signals table.", body = [Signal]),
        (status = 500, description = "Database error", body = serde_json::Value)
    )
)]
pub async fn list_signals(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as::<sqlite::Sqlite, Signal>("SELECT * FROM signals")
        .fetch_all(&state.db)
        .await
    {
        Ok(signals) => (StatusCode::OK, Json(signals)).into_response(),
        Err(e) => {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("failed to query database {}", e),
                    "http/handlers.rs",
                    141,
                ),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Database error"})),
            )
                .into_response()
        }
    }
}

/// Crawl all tickers for klines.
///
/// Returns a 200 if the job is submitted.
#[utoipa::path(
    get,
    path = "/crawl",
    tag = "candlescyther",
    params(
        CrawlQuery
    ),
    responses(
        (status = 200, description = "Job submitted"),
        (status = 400, description = "Tickers is required", body = serde_json::Value),
        (status = 500, description = "Job runner error", body = serde_json::Value),
    )
)]
pub async fn crawl_klines(
    State(state): State<AppState>,
    query: Query<CrawlQuery>,
) -> impl IntoResponse {
    if query.tickers.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "tickers: [String] is required."})),
        )
            .into_response();
    }

    let mut jobs = vec![];
    for ticker in &query.tickers {
        jobs.push((
            JobType::CrawlPrice,
            json!(CrawlPricePayload {
                ticker: ticker.to_string(),
                url: format!("https://54.push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35106707668456928451_1695010059469&secid={}&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=102&fqt=1&beg=0&end=20500101&lmt=1200&_=1695010059524", ticker)
            })
        ));
    }

    if let Err(e) = state.runner.repository.create_jobs(jobs).await {
        logit(
            &state,
            LogEntry::new(
                LogLevel::Error,
                "failed to create_jobs in crawl_klines",
                "http/handlers.rs",
                180,
            ),
        )
        .await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": format!("job runner error {}", e.to_string())})),
        )
            .into_response();
    }

    tokio::spawn(async move {
        if let Err(e) = state.runner.run().await {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("runner error: {}", e),
                    "http/handlers.rs",
                    238,
                ),
            )
            .await;
        }
    });

    (StatusCode::OK).into_response()
}

#[derive(Deserialize, IntoParams)]
pub struct CrawlQuery {
    #[serde(deserialize_with = "deserialize_comma_separated")]
    pub tickers: Vec<String>,
}

fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

/// Trigger compute signals job.
///
/// Returns OK if job submitted.
#[utoipa::path(
    get,
    path = "/run/signals",
    tag = "candlescyther",
    responses(
        (status = 200, description = "Job submitted"),
        (status = 500, description = "Job runner error", body = serde_json::Value),
    )
)]
pub async fn compute_signals(State(state): State<AppState>) -> impl IntoResponse {
    let job = (JobType::ComputeSignal, json!(ComputeSignalPayload {}));

    if let Err(e) = state.runner.repository.create_jobs(vec![job]).await {
        logit(
            &state,
            LogEntry::new(
                LogLevel::Error,
                "failed to create_jobs in compute_signals",
                "http/handlers.rs",
                199,
            ),
        )
        .await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": format!("job runner error {}", e.to_string())})),
        )
            .into_response();
    }

    tokio::spawn(async move {
        if let Err(e) = state.runner.run().await {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("runner error: {}", e),
                    "http/handlers.rs",
                    218,
                ),
            )
            .await;
        }
    });

    (StatusCode::OK).into_response()
}

/// List all klines per ticker.
///
/// Returns all klines for give ticker.
#[utoipa::path(
    get,
    path = "/klines",
    tag = "candlescyther",
    params(
        KlineQuery   
    ),
    responses(
        (status = 200, description = "List all klines for the ticker", body = [Kline]),
        (status = 400, description = "Ticker is required", body = serde_json::Value),
        (status = 500, description = "Database error", body = serde_json::Value)
    )
)]
pub async fn list_klines(State(state): State<AppState>, query: Query<KlineQuery>) -> impl IntoResponse {
    if query.ticker.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "ticker: String is required."})),
        )
            .into_response();
    }

    match sqlx::query_as::<sqlite::Sqlite, Kline>("SELECT * FROM klines WHERE k_ticker = ? ORDER BY k_date ASC")
        .bind(&query.ticker)
        .fetch_all(&state.db)
        .await
    {
        Ok(signals) => (StatusCode::OK, Json(signals)).into_response(),
        Err(e) => {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("failed to query database {}", e),
                    "http/handlers.rs",
                    141,
                ),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Database error"})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct KlineQuery {
    pub ticker: String,
}
