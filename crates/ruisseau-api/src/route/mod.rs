use crate::{oauth, route};
use aide::axum::routing::{get, post};
use aide::axum::ApiRouter;
use aide::redoc::Redoc;
use axum::{middleware, Extension};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub mod openapi;
pub mod repository;
pub mod user;

pub fn app(pool: PgPool) -> ApiRouter {
    let public = ApiRouter::new()
        .api_route("/health", get(|| async { "Pong" }))
        .api_route("/users", post(route::user::create))
        .layer(Extension(pool.clone()))
        .layer(TraceLayer::new_for_http());

    let authenticated = ApiRouter::new()
        .api_route("/users/ssh_key/add", post(route::user::register_ssh_key))
        .api_route("/repositories", post(route::repository::init_repository))
        .layer(Extension(pool.clone()))
        .layer(TraceLayer::new_for_http())
        .route_layer(middleware::from_fn(oauth::service::auth));

    public.merge(authenticated)
}

pub fn docs_router() -> ApiRouter {
    ApiRouter::new()
        .route("/", Redoc::new("/docs/openapi.json").axum_route())
        .route("/openapi.json", get(route::openapi::serve_api))
        .layer(TraceLayer::new_for_http())
}
