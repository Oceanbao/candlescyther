use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};
use sqlx::sqlite;

use utoipa::{IntoParams, ToSchema};

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    application::{
        handlers::handler_create_stock::CreateStockPayload,
        model::{Job, JobType},
    },
    domain::model::{Kline, Signal, Stock, User},
    infra::{
        http::AppState,
        logging::{LogEntry, LogLevel, logit},
    },
};

#[derive(Serialize, Deserialize, ToSchema)]
enum ApiError {
    #[schema(example = "failed to retrieve from database")]
    DatabaseError(String),
    #[schema(example = "id = 1")]
    NotFound(String),
    #[schema(example = "require query param x")]
    MissingInput(String),
    #[schema(example = "missing api key")]
    Unauthorized(String),
    #[schema(example = "job runner error")]
    RunnerError(String),
}

pub fn create_routes_api(app_state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        // /logs
        .routes(routes!(list_logs))
        // /jobs
        .routes(routes!(list_jobs))
        // /signals
        .routes(routes!(list_signals))
        // /stocks GET, POST, DELETE
        .routes(routes!(create_stocks, list_stocks, delete_stock))
        // /klines?ticker=a
        .routes(routes!(list_klines))
        // /trigger/all
        .routes(routes!(update_all))
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

// FIX: create better ErrorResponse to frontend.

/// List all jobs.
///
/// Returns all jobs.
#[utoipa::path(
    get,
    path = "/jobs",
    tag = "candlescyther",
    responses(
        (status = 200, description = "List all jobs from jobs table.", body = [Job]),
        (status = 500, description = "Database error", body = ApiError)
    )
)]
pub async fn list_jobs(State(state): State<AppState>) -> impl IntoResponse {
    match state.runner.repo_job.get_jobs_all().await {
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
                Json(ApiError::DatabaseError(e.to_string())),
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
        (status = 500, description = "Database error", body = ApiError)
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
                Json(ApiError::DatabaseError(e.to_string())),
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
        (status = 500, description = "Database error", body = ApiError)
    )
)]
pub async fn list_signals(State(state): State<AppState>) -> impl IntoResponse {
    match state.runner.repo_domain.get_signals_all().await {
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
                Json(ApiError::DatabaseError(e.to_string())),
            )
                .into_response()
        }
    }
}

/// List all stocks.
///
/// Returns all stocks.
#[utoipa::path(
    get,
    path = "/stocks",
    tag = "candlescyther",
    responses(
        (status = 200, description = "List all stocks from stocks table.", body = [Stock]),
        (status = 500, description = "Database error", body = ApiError)
    )
)]
pub async fn list_stocks(State(state): State<AppState>) -> impl IntoResponse {
    match state.runner.repo_domain.get_stock_all().await {
        Ok(stocks) => (StatusCode::OK, Json(stocks)).into_response(),
        Err(e) => {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("failed to query database {}", e),
                    "http/handlers.rs",
                    211,
                ),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::DatabaseError(e.to_string())),
            )
                .into_response()
        }
    }
}

/// Create stocks with meta,klines,signals.
///
/// Returns a 200 if the job is submitted.
#[utoipa::path(
    post,
    path = "/stocks",
    tag = "candlescyther",
    responses(
        (status = 200, description = "Job submitted"),
        (status = 400, description = "Tickers is required", body = ApiError),
        (status = 500, description = "Job runner error", body = ApiError),
    )
)]
pub async fn create_stocks(
    State(state): State<AppState>,
    Json(req_body): Json<CreateStockRequest>,
) -> impl IntoResponse {
    let tickers: Vec<String> = req_body
        .tickers
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    if tickers.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError::MissingInput(
                "`ticker` field required in body".to_string(),
            )),
        )
            .into_response();
    }

    let mut jobs = vec![];
    for ticker in &tickers {
        jobs.push(Job::new(
            JobType::CreateStock,
            json!(CreateStockPayload {
                ticker: ticker.to_string(),
                update: false,
            }),
        ));
    }

    if let Err(e) = state.runner.repo_job.create_jobs(jobs).await {
        logit(
            &state,
            LogEntry::new(
                LogLevel::Error,
                "failed to create_jobs in create_stocks",
                "http/handlers.rs",
                214,
            ),
        )
        .await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::RunnerError(e.to_string())),
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

#[derive(Deserialize, ToSchema)]
pub struct CreateStockRequest {
    pub tickers: String,
}

#[derive(Deserialize, ToSchema)]
pub struct DemoQuery {
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

/// Delete stock.
#[utoipa::path(
    delete,
    path = "/stocks",
    params(
        DeleteStockQuery,
    ),
    tag = "candlescyther",
    responses(
        (status = 200, description = "Delete stock and its records"),
        (status = 400, description = "Ticker is required", body = ApiError),
        (status = 500, description = "Database error", body = ApiError)
    )
)]
pub async fn delete_stock(
    State(state): State<AppState>,
    query: Query<DeleteStockQuery>,
) -> impl IntoResponse {
    if query.ticker.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError::MissingInput("missing param".to_string())),
        )
            .into_response();
    }
    match state.runner.repo_domain.delete_stock(&query.ticker).await {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => {
            logit(
                &state,
                LogEntry::new(
                    LogLevel::Error,
                    format!("failed to query database {}", e),
                    "http/handlers.rs",
                    211,
                ),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::DatabaseError(e.to_string())),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct DeleteStockQuery {
    ticker: String,
}

/// List all klines per ticker.
///
/// Returns all klines for give ticker.
#[utoipa::path(
    get,
    path = "/klines",
    tag = "candlescyther",
    params(
        KlineQuery,
    ),
    responses(
        (status = 200, description = "List all klines for the ticker", body = [Kline]),
        (status = 400, description = "Ticker is required", body = ApiError),
        (status = 500, description = "Database error", body = ApiError)
    )
)]
pub async fn list_klines(
    State(state): State<AppState>,
    query: Query<KlineQuery>,
) -> impl IntoResponse {
    if query.ticker.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError::MissingInput("missing param".to_string())),
        )
            .into_response();
    }

    match sqlx::query_as::<sqlite::Sqlite, Kline>(
        "SELECT * FROM klines WHERE k_ticker = ? ORDER BY k_date ASC",
    )
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
                Json(ApiError::DatabaseError(e.to_string())),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct KlineQuery {
    pub ticker: String,
}

/// Trigger update of all.
///
/// Returns.
#[utoipa::path(
    get,
    path = "/trigger/all",
    tag = "candlescyther",
    params(
        TriggerQuery,
    ),
    responses(
        (status = 200, description = "Trigger is init."),
        (status = 400, description = "Missing code", body = ApiError),
        (status = 500, description = "Database error", body = ApiError)
    )
)]
pub async fn update_all(
    State(state): State<AppState>,
    query: Query<TriggerQuery>,
) -> impl IntoResponse {
    // FIX: refine security
    if query.code.is_empty() || query.code != "1l0veu" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError::MissingInput("missing code".to_string())),
        )
            .into_response();
    }

    let tickers: Vec<String> = match state.runner.repo_domain.get_stock_all().await {
        Ok(tickers) => tickers.iter().map(|t| t.ticker.clone()).collect(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::DatabaseError(e.to_string())),
            )
                .into_response();
        }
    };

    let mut jobs = vec![];
    for ticker in &tickers {
        jobs.push(Job::new(
            JobType::CreateStock,
            json!(CreateStockPayload {
                ticker: ticker.to_string(),
                update: true,
            }),
        ));
    }

    if let Err(e) = state.runner.repo_job.create_jobs(jobs).await {
        logit(
            &state,
            LogEntry::new(
                LogLevel::Error,
                "failed to create_jobs in update_all",
                "http/handlers.rs",
                491,
            ),
        )
        .await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::RunnerError(e.to_string())),
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
                    510,
                ),
            )
            .await;
        }
    });

    (StatusCode::OK).into_response()
}

#[derive(Deserialize, IntoParams)]
pub struct TriggerQuery {
    pub code: String,
}
