use askama::Template;
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

pub mod login;
pub mod repositories;
pub mod repository;

pub struct HtmlTemplate<T>(T);

pub fn view_router() -> Router {
    Router::new()
        .nest("/", repository::routes())
        .route("/repo", get(repositories::list))
        .route("/login/callback", get(login::callback))
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
