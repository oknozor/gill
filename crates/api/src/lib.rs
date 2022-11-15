use aide::axum::routing::{get, post};
use aide::axum::ApiRouter;
use aide::redoc::Redoc;
use aide::openapi::{Info, OpenApi};
use axum::Extension;
use sqlx::PgPool;
use std::net::SocketAddr;
use once_cell::sync::Lazy;
use tower_http::trace::TraceLayer;
use crate::settings::Settings;

pub mod error;
pub mod route;
pub mod oauth;
pub mod settings;

pub const SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::get().expect("Config error"));

pub fn app(pool: PgPool) -> ApiRouter {

    ApiRouter::new()
        .route("/docs", Redoc::new("/openapi.json").axum_route())
        .route("/openapi.json", get(route::openapi::serve_api))
        .api_route("/", get(|| async { "Hello, World!" }))
        .api_route("/users", post(route::user::create))
        .api_route("/users/:id", get(route::user::by_id))
        .api_route("/repository/init", post(route::repository::init_repository))
        .api_route("/ssh_key/register", post(route::user::register_ssh_key))
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
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
            app(db)
                .finish_api(&mut api)
                .layer(Extension(api))
                .into_make_service(),
        )
        .await?;

    Ok(())
}
