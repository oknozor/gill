use askama::Template;
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, IntoMakeService};
use axum::{Extension, Router, RouterService};
use sqlx::PgPool;

pub mod blob;
pub mod repositories;
pub mod tree;

pub struct HtmlTemplate<T>(T);

pub fn view_router() -> Router {
    Router::new()
        .route("/repo", get(repositories::list))
        .route("/:owner/:repository/tree/:branch/*tree", get(tree::tree))
        .route("/:owner/:repository/tree/:branch", get(tree::tree_root))
        .route("/:owner/:repository/blob/:branch/*blob", get(blob::blob))
}

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
