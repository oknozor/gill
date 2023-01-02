use crate::state::AppState;
use crate::view::repository::pull_request::comment::comment;
use crate::view::repository::pull_request::compare::compare;
use crate::view::repository::pull_request::create::create;
use crate::view::repository::pull_request::list_view::list_view;
use crate::view::repository::pull_request::view::{close, merge, view};
use axum::routing::get;
use axum::Router;

pub mod comment;
pub mod compare;
pub mod create;
pub mod list_view;
pub mod view;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:owner/:repository/pulls", get(list_view))
        .route("/:owner/:repository/pulls/:number", get(view))
        .route("/:owner/:repository/pulls/:number/comment", get(comment))
        .route("/:owner/:repository/pulls/:number/merge", get(merge))
        .route(
            "/:owner/:repository/pulls/:number/rebase",
            get(view::rebase),
        )
        .route("/:owner/:repository/pulls/:number/close", get(close))
        .route("/:owner/:repository/pulls/create", get(create))
        .route("/:owner/:repository/compare", get(compare))
}
