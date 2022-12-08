use axum::http::StatusCode;
use axum::response::IntoResponse;

use async_session::MemoryStore;
use axum::{Extension, Router};
use gill_api::api::router;
use gill_api::instance::Instance;
use gill_api::oauth::{oauth_client, AppState};
use gill_api::syntax::{load_syntax, load_theme};
use gill_api::{api, apub, view, SETTINGS};
use gill_ipc::listener::IPCListener;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "gill_ipc=debug,gill_git=debug,gill_api=debug,tower_http=debug".into()
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
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Serving at: {}", addr);
    let instance = Instance::new("localhost:3000".to_string(), db).unwrap();
    Instance::listen(&instance).await?;

    Ok(())
}
