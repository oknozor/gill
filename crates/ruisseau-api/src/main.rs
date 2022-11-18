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
    println!("it gets env");
    dotenvy::dotenv().ok();
    println!("Ho no");
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

    println!("it migrates");
    sqlx::migrate!().run(&pool).await?;
    println!("Oh no");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
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

    println!("it serve");
    axum::Server::bind(&addr)
        .serve(
            route::app()
                .nest("/docs", route::docs_router())
                .finish_api(&mut api)
                .layer(Extension(api))
                .route("/repo", get(app::repositories_view::list))
                .route("/blob/:owner/:repository/:tree", get(app::tree_view::tree))
                .layer(Extension(db))
                .into_make_service(),
        )
        .await?;

    Ok(())
}
