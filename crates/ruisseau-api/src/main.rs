use aide::openapi::{Info, OpenApi};
use axum::routing::get;
use axum::Extension;
use ruisseau_api::{app, route};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "ruisseau_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ruisseau:ruisseau@localhost/ruisseau".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    sqlx::migrate!().run(&pool).await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    serve(pool, addr).await?;

    Ok(())
}

pub async fn serve(db: PgPool, addr: SocketAddr) -> eyre::Result<()> {
    let mut api = OpenApi {
        info: Info {
            description: Some("Legit Api".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    axum::Server::bind(&addr)
        .serve(
            route::app()
                .nest("/docs", route::docs_router())
                .finish_api(&mut api)
                .layer(Extension(api))
                .route("/repo", get(app::repositories_pages::list))
                .layer(Extension(db))
                .into_make_service(),
        )
        .await?;

    Ok(())
}
