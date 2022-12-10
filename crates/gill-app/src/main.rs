use gill_app::instance::Instance;
use gill_settings::SETTINGS;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "gill_ipc=debug,gill_git=debug,gill_app=debug,tower_http=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let connection_url = &SETTINGS.database_url();
    tracing::debug!("Connecting to {connection_url}");
    let db = PgPoolOptions::new()
        .max_connections(10)
        .idle_timeout(Duration::from_secs(3))
        .connect(connection_url)
        .await
        .expect("can connect to database");

    tracing::debug!("Loading config: {:?}", *SETTINGS);
    let instance = Instance::new(SETTINGS.domain.to_string(), db).unwrap();
    Instance::listen(&instance).await?;

    Ok(())
}
