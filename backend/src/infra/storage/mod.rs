use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::{env, time::Duration};
use tracing::info;

use sqlx::SqlitePool;

pub mod repo_domain_sqlite;
pub mod repo_job_sqlite;

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Database, anyhow::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        // FIX: clean up this logic and hardcode
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

        info!("Database initialized");

        Ok(Database { pool })
    }
}

/*
Dynamic query building.

async fn search_users(
    pool: &PgPool,
    name_filter: Option<&str>,
    email_filter: Option<&str>
) -> Result<Vec<User>, sqlx::Error> {
    let mut query = String::from("SELECT id, name, email FROM users WHERE 1=1");
    let mut params = Vec::new();
    let mut param_count = 0;

    if let Some(name) = name_filter {
        param_count += 1;
        query.push_str(&format!(" AND name LIKE ${}", param_count));
        params.push(format!("%{}%", name));
    }

    if let Some(email) = email_filter {
        param_count += 1;
        query.push_str(&format!(" AND email LIKE ${}", param_count));
        params.push(format!("%{}%", email));
    }

    let mut query_builder = sqlx::query_as::<_, User>(&query);

    for param in params {
        query_builder = query_builder.bind(param);
    }

    query_builder.fetch_all(pool).await
}jk
*/
