use backend::{
    app::{AppState, User},
    logging::{LogEntry, LogLevel, logit},
};
use serde_json::{Value, json};
use std::{env, time::Duration};
use tracing::info;

use axum::{Json, Router, extract::State, routing::get};
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{self, SqlitePoolOptions},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // if !std::path::Path::new("db").exists() {
    //     fs::create_dir("db")?;
    // }
    // let path_buf = env::current_dir()?;
    // info!("CWD --------------------------- : {}", path_buf.display());

    let database_url = "sqlite:///db/database.db?mode=rwc";

    match sqlx::Sqlite::database_exists(database_url).await {
        Ok(exist) => {
            if !exist {
                match sqlx::Sqlite::create_database(database_url).await {
                    Ok(_) => info!("Database created."),
                    Err(e) => info!("Database creation failed: {}", e),
                }
            } else {
                info!("Database exist!");
            }
        }
        Err(e) => info!("Database checking failed: {}", e),
    }

    // Initialize database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    if let Err(e) = sqlx::query("INSERT INTO user (user_name, user_role) VALUES (?, ?)")
        .bind("ocean")
        .bind("admin")
        .execute(&pool)
        .await
    {
        tracing::error!("Failed to insert ocean user: {}", e);
    }

    info!("Database initialized");

    // Build our application with a single route
    let app = Router::new()
        .route("/check", get(check_handler))
        .route("/logs", get(log_handler))
        .with_state(AppState { db: pool });

    // Start the cron job in a separate task
    // let cron_state = shared_state.clone();
    // tokio::spawn(async move {
    //     if let Err(e) = start_cron_job(cron_state).await {
    //         tracing::error!("Cron job error: {}", e);
    //     }
    // });

    // Run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("server up and listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler for the /check endpoint
async fn check_handler(State(state): State<AppState>) -> Json<Value> {
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

async fn log_handler(State(state): State<AppState>) -> Json<Value> {
    logit(
        &state,
        LogEntry::new(LogLevel::Debug, "pre-select", "log_handler", 93),
    )
    .await;

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
