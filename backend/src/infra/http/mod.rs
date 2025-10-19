pub mod handler;

use sqlx::SqlitePool;
use std::fs;
use tracing::info;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    application::runner::JobRunner,
    infra::{http::handler::create_routes_api, storage::Database},
};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub runner: JobRunner,
}

#[derive(OpenApi)]
#[openapi(
        tags(
            (name = "candlescyther", description = "Reap fat candles.")
        )
    )]
struct ApiDoc;

pub async fn init_server(db: Database, runner: JobRunner) -> anyhow::Result<()> {
    // let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
    //     |request: &axum::extract::Request<_>| {
    //         let uri = request.uri().to_string();
    //         tracing::info_span!("http_request", method = ?request.method(), uri)
    //     },
    // );

    let app_state = AppState {
        db: db.pool,
        runner,
    };

    let routes_api = create_routes_api(app_state);
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", routes_api)
        .split_for_parts();

    fs::write("./openapi.json", api.to_pretty_json().unwrap())?;

    // Build our application with a single route
    // let app = Router::new()
    //     .route("/check", get(check_handler))
    //     .route("/logs", get(log_handler))
    //     .with_state(AppState { db: pool });

    // Start the cron job in a separate task
    // let cron_state = shared_state.clone();
    // tokio::spawn(async move {
    //     if let Err(e) = start_cron_job(cron_state).await {
    //         tracing::error!("Cron job error: {}", e);
    //     }
    // });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("server up and listening on {}", listener.local_addr()?);
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}
