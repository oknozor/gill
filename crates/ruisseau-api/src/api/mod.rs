use crate::oauth;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub mod repository;
pub mod user;

pub fn rest_api() -> Router {
    /// FIXME: properly mount /api/v1 as a prefix
    let public = Router::new()
        .route("/health", get(|| async { "Pong" }))
        .route("/users", post(user::create))
        .route("/repositories", get(repository::list));

    let authenticated = Router::new()
        .route("/users/ssh_key/add", post(user::register_ssh_key))
        .route("/repositories", post(repository::init))
        .route_layer(middleware::from_fn(oauth::service::auth));

    public.merge(authenticated)
}
