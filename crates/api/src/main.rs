#![feature(trivial_bounds)]
use std::net::SocketAddr;
use std::time::Duration;
use aide::{
    axum::{ routing::{get, post}, ApiRouter },
    openapi::{Info, OpenApi},
};
use axum::Extension;
use aide::axum_redoc::Redoc;
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use route::repository;

mod route;
mod error;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://legit:legit@localhost/legit".to_string());

    // setup connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");


    sqlx::migrate!().run(&pool).await?;

    let app = ApiRouter::with_state(pool)
        .nest("/docs", Redoc::setup("/openapi.json").into())
        .route("/openapi.json", get(route::openapi::serve_api))
        .api_route("/", get(|| async { "Hello, World!" }))
        .api_route("/user", post(route::user::create))
        .api_route("/repository/init", post(repository::init_repository))
        .api_route("/ssh_key/register", post(route::user::register_ssh_key))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    let mut api = OpenApi {
        info: Info {
            description: Some("Legit Api".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    axum::Server::bind(&addr)
        .serve(app
            .finish_api(&mut api)
            .layer(Extension(api))
            .into_make_service())
        .await?;

    Ok(())
}

