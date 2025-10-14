use backend::{
    infra::http::{handler::create_routes, server::AppState},
    job::{
        handler::{ComputeSignalHandler, CrawlPriceHandler, JobHandlerRegistry},
        repository::SqliteJobRepository,
        runner::JobRunner,
    },
};
use std::{env, fs, sync::Arc, time::Duration};
use tracing::info;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // FIX: clean up
    let db_filepath = "./db/database.db";
    match env::var("RESET_DB") {
        Ok(_) => {
            // Delete the existing database file
            if std::path::Path::new(db_filepath).exists() {
                std::fs::remove_file(db_filepath)?;
                info!("Deleted existing database file: {}", db_filepath);
            }
        }
        Err(_) => {
            info!("RESET_DB is not set.");
        }
    }

    match sqlx::Sqlite::database_exists(&database_url).await {
        Ok(exist) => {
            if !exist {
                match sqlx::Sqlite::create_database(&database_url).await {
                    Ok(_) => info!("Database created."),
                    Err(e) => info!("Database creation failed: {}", e),
                }
            } else {
                info!("Database exists!");
            }
        }
        Err(e) => info!("Database checking failed: {}", e),
    }

    // Initialize database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    if let Err(e) = sqlx::query!(
        "INSERT INTO users (user_name, user_role) VALUES (?, ?)",
        "ocean",
        "admin"
    )
    .execute(&pool)
    .await
    {
        tracing::error!("Failed to insert ocean user: {}", e);
    }

    info!("Database initialized");

    #[derive(OpenApi)]
    #[openapi(
        tags(
            (name = "candlescyther", description = "Reap fat candles.")
        )
    )]
    struct ApiDoc;

    let crawltest_handler = CrawlPriceHandler { pool: pool.clone() };
    let compute_signal_handler = ComputeSignalHandler { pool: pool.clone() };

    let mut handler_registry = JobHandlerRegistry::new();
    handler_registry.register_handler(Arc::new(crawltest_handler));
    handler_registry.register_handler(Arc::new(compute_signal_handler));

    let concurrency = 3;
    let wait_ms = 1000;
    let batch_size = concurrency;
    let sqlite_repository = SqliteJobRepository { pool: pool.clone() };

    let runner = JobRunner::new(
        Arc::new(sqlite_repository),
        Arc::new(handler_registry),
        concurrency,
        wait_ms,
        batch_size,
    );

    let store = AppState { db: pool, runner };

    let routes = create_routes(store);
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", routes)
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
