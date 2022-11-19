use std::io;
use aide::openapi::{Info, OpenApi};
use axum::routing::get;
use axum::{Extension, Router};
use ruisseau_api::{app, route, SETTINGS};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::time::Duration;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "ruisseau_git=debug,ruisseau_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@postgres/ruisseau".to_string());

    tracing::debug!("Connecting to {db_connection_str}");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    tracing::debug!("Running database migrations");
    sqlx::migrate!().run(&pool).await?;

    tracing::debug!("Loading config: {:?}", *SETTINGS);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Serving at: {}", addr);
    serve(pool, addr).await?;

    Ok(())
}

pub async fn serve(db: PgPool, addr: SocketAddr) -> eyre::Result<()> {
    let mut api = OpenApi {
        info: Info {
            description: Some("Ruisseau Api".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let serve_dir = axum::routing::get_service(ServeDir::new("assets")).handle_error(handle_error);
    let assets_router = Router::new().nest_service("/", serve_dir);

    axum::Server::bind(&addr)
        .serve(
            route::app()
                .nest("/assets", assets_router.into())
                .layer(TraceLayer::new_for_http())
                .nest("/docs", route::docs_router())
                .finish_api(&mut api)
                .layer(Extension(api))
                .route("/repo", get(app::repositories_view::list))
                .route("/:owner/:repository/tree/:branch/*tree", get(app::tree_view::tree))
                .route("/:owner/:repository/tree/:branch", get(app::tree_view::tree_root))
                .route("/:owner/:repository/blob/:branch/*blob", get(app::blob_view::blob))
                .layer(Extension(db))
                .into_make_service(),
        )
        .await?;

    Ok(())
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

