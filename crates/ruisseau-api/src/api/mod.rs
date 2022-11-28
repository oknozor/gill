use crate::oauth;
use axum::{middleware, Router, routing::{get, post}};

pub mod repository;
pub mod user;

pub fn rest_api() -> Router {
    /// FIXME: properly mount /api/v1 as a prefix
    let public = Router::new()
        .route("/api/v1/health", get(|| async { "Pong" }))
        .route("/api/v1/users", post(user::create))
        .route("/api/v1/repositories", get(repository::list));

    let authenticated = Router::new()
        .route("/api/v1/users/ssh_key/add", post(user::register_ssh_key))
        .route("/api/v1/repositories", post(repository::init))
        .route_layer(middleware::from_fn(oauth::service::auth));

    public.merge(authenticated)
}