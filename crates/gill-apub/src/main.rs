use gill_apub::error;
use gill_apub::instance::Instance;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[actix_rt::main]
async fn main() -> Result<(), error::Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "gill_apub=debug,gill_db=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/gill".to_string());

    tracing::debug!("Connecting to {db_connection_str}");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .idle_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    tracing::debug!("Creating activity pub instance for localhost:3000",);

    let instance = Instance::new("localhost:3000".to_string(), pool)?;
    Instance::listen(&instance).await?;
    Ok(())
}
