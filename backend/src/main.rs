use serde_json::{Value, json};
use std::env;
use tracing::info;

use axum::{Json, Router, extract::State, routing::get};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initialize database
    let pool = SqlitePool::connect(&database_url).await?;
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

#[derive(Clone)]
struct AppState {
    pub db: SqlitePool,
}

// Handler for the /check endpoint
async fn check_handler(State(state): State<AppState>) -> Json<Value> {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(result) => {
            let res = format!("{result:#?}");
            Json(json!({
                "status": "ok",
                "database": "connected",
                "data": res,
            }))
        }
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
