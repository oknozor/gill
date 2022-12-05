use axum::http::StatusCode;
use axum::response::IntoResponse;

use async_session::MemoryStore;
use axum::{Extension, Router};
use gill_api::api::rest_api;
use gill_api::oauth::{oauth_client, AppState};
use gill_api::syntax::{load_syntax, load_theme};
use gill_api::{view, SETTINGS};
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

#[tokio::main]
async fn main() -> eyre::Result<()> {
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
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .idle_timeout(Duration::from_secs(3))
        .connect(connection_url)
        .await
        .expect("can connect to database");

    tracing::debug!("Running database migrations");

    tracing::debug!("Loading config: {:?}", *SETTINGS);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Serving at: {}", addr);
    serve(pool, addr).await?;

    Ok(())
}

pub async fn serve(db: PgPool, addr: SocketAddr) -> eyre::Result<()> {
    let serve_dir = axum::routing::get_service(ServeDir::new("assets")).handle_error(handle_error);
    /// FIXME: not suitable for production replace with redis maybe
    let store = MemoryStore::new();
    let oauth_client = oauth_client();
    let syntax_set = load_syntax();
    let theme = load_theme();
    let app_state = AppState {
        store,
        oauth_client,
        syntax_set,
        theme,
    };

    let api = axum::Server::bind(&addr).serve(
        Router::new()
            .nest("/api/v1/", rest_api())
            .nest_service("/assets/", serve_dir)
            .nest_service("/", view::view_router(app_state))
            .layer(TraceLayer::new_for_http())
            .layer(Extension(db))
            .into_make_service(),
    );

    let ipc = IPCListener;
    let _ = tokio::join!(api, ipc.listen());

    Ok(())
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
