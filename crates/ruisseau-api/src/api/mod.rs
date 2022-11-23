use crate::oauth;
use aide::axum::routing::{get, post};
use aide::axum::ApiRouter;
use aide::redoc::Redoc;
use axum::middleware;

pub mod openapi;
pub mod repository;
pub mod user;

pub fn rest_api() -> ApiRouter {
    /// FIXME: properly mount /api/v1 as a prefix
    let public = ApiRouter::new()
        .api_route("/api/v1/health", get(|| async { "Pong" }))
        .api_route("/api/v1/users", post(user::create))
        .api_route("/api/v1/repositories", get(repository::list));

    let authenticated = ApiRouter::new()
        .api_route("/api/v1/users/ssh_key/add", post(user::register_ssh_key))
        .api_route("/api/v1/repositories", post(repository::init))
        .route_layer(middleware::from_fn(oauth::service::auth));

    public.merge(authenticated)
}

pub fn docs_router() -> ApiRouter {
    ApiRouter::new()
        .route(
            "/api/v1/docs",
            Redoc::new("/docs/openapi.json").axum_route(),
        )
        .route("/api/v1/openapi.json", get(openapi::serve_api))
}
