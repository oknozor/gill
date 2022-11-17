use crate::oauth;
use aide::axum::routing::{get, post};
use aide::axum::ApiRouter;
use aide::redoc::Redoc;
use axum::{middleware, Extension};
use schemars::JsonSchema;
use serde::Deserialize;
use sqlx::PgPool;
use std::num::NonZeroI64;
use tower_http::trace::TraceLayer;

pub mod openapi;
pub mod repository;
pub mod user;

#[derive(Deserialize, JsonSchema)]
pub struct Pagination {
    page: NonZeroI64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: NonZeroI64::new(1).unwrap(),
        }
    }
}

pub fn app(pool: PgPool) -> ApiRouter {
    let public = ApiRouter::new()
        .api_route("/health", get(|| async { "Pong" }))
        .api_route("/users", post(user::create))
        .api_route("/repositories", get(repository::list))
        .layer(Extension(pool.clone()))
        .layer(TraceLayer::new_for_http());

    let authenticated = ApiRouter::new()
        .api_route("/users/ssh_key/add", post(user::register_ssh_key))
        .api_route("/repositories", post(repository::init))
        .layer(Extension(pool.clone()))
        .layer(TraceLayer::new_for_http())
        .route_layer(middleware::from_fn(oauth::service::auth));

    public.merge(authenticated)
}

pub fn docs_router() -> ApiRouter {
    ApiRouter::new()
        .route("/", Redoc::new("/docs/openapi.json").axum_route())
        .route("/openapi.json", get(openapi::serve_api))
        .layer(TraceLayer::new_for_http())
}
