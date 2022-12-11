use crate::state::AppState;
use axum::routing::get;
use axum::Router;

pub mod profile;
pub mod settings;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:owner", get(profile::user_view))
        .route("/:owner/", get(profile::user_view))
        .route("/settings/profile", get(settings::settings))
}
