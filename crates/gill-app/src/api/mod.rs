use crate::oauth;
use crate::state::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub mod repository;
pub mod user;

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(|| async { "Pong" }))
        .route("/health/", get(|| async { "Pong" }))
        .route("/users", post(user::create))
        .route("/users/", post(user::create));

    let authenticated = Router::new()
        .route("/users/ssh_key/add", post(user::register_ssh_key))
        .route("/users/ssh_key/add/", post(user::register_ssh_key))
        .route("/repositories/create", post(repository::init))
        .route("/repositories/create/", post(repository::init))
        .route_layer(middleware::from_fn(oauth::service::auth));

    public.merge(authenticated).with_state(state)
}
