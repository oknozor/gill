use crate::oauth;
use crate::oauth::AppState;
use askama::Template;
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

pub mod index;
pub mod repositories;
pub mod repository;

pub struct HtmlTemplate<T>(T);

pub fn view_router(app_state: AppState) -> Router {
    Router::new()
        .merge(repository::routes())
        .route("/", get(index::index))
        .route("/auth/ruisseau/", get(oauth::openid_auth))
        .route("/auth/authorized/", get(oauth::login_authorized))
        .route("/logout/", get(oauth::logout))
        .route("/repo/", get(repositories::list))
        .with_state(app_state)
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
