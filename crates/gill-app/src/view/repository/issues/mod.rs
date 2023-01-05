use crate::state::AppState;
use crate::view::repository::issues::close::close;
use crate::view::repository::issues::comment::comment;
use crate::view::repository::issues::create::create;
use crate::view::repository::issues::list_view::list_view;
use crate::view::repository::issues::view::view;
use axum::routing::get;
use axum::Router;

pub mod close;
pub mod comment;
pub mod create;
pub mod list_view;
pub mod view;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:owner/:repository/issues", get(list_view))
        .route("/:owner/:repository/issues/:number", get(view))
        .route("/:owner/:repository/issues/:number/comment", get(comment))
        .route("/:owner/:repository/issues/:number/close", get(close))
        .route("/:owner/:repository/issues/create", get(create))
}
