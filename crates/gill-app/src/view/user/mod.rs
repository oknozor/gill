use crate::state::AppState;
use axum::routing::get;
use axum::Router;

pub mod profile;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:owner", get(profile::get_profile))
        .route("/:owner/", get(profile::get_profile))
}
