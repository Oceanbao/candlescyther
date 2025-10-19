use backend::{
    application::init_runner,
    infra::{http::init_server, storage::Database},
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database = Database::new().await?;

    let runner = init_runner(&database);

    init_server(database, runner).await
}
