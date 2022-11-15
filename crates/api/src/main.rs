use std::net::SocketAddr;
use aide::{
    axum::{ routing::{get, post}, ApiRouter },
    openapi::{Info, OpenApi},
};
use axum::Extension;
use aide::axum_redoc::Redoc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use route::repository;
use route::user;

mod route;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = ApiRouter::new()
        .nest("/docs", Redoc::setup("/openapi.json").into())
        .route("/openapi.json", get(route::openapi::serve_api))
        .api_route("/", get(|| async { "Hello, World!" }))
        .api_route("/repository/init", post(repository::init_repository))
        .api_route("/ssh_key/register", post(user::register_ssh_key))
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
            // Expose the documentation to the handlers.
            .layer(Extension(api))
            .into_make_service())
        .await
        .unwrap();
}

