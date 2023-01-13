use crate::oauth;

use crate::state::AppState;
use crate::view::follow::follow_form;
use askama::{DynTemplate, Template};
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

pub mod component;
pub mod dto;
mod filters;
pub mod follow;
pub mod index;
pub mod repository;
pub mod user;

pub struct HtmlTemplate<T>(T);

pub struct DynHtmlTemplate<T>(T);

impl<T> HtmlTemplate<T> {
    pub fn inner(self) -> T {
        self.0
    }
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .merge(repository::routes())
        .merge(user::routes())
        .route("/", get(index::index))
        .route("/auth/gill/", get(oauth::openid_auth))
        .route("/auth/gill", get(oauth::openid_auth))
        .route("/auth/authorized/", get(oauth::login_authorized))
        .route("/auth/authorized", get(oauth::login_authorized))
        .route("/logout/", get(oauth::logout))
        .route("/follow_user", get(follow_form))
        .route("/follow_user/", get(follow_form))
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
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for DynHtmlTemplate<Box<dyn DynTemplate>> {
    fn into_response(self) -> Response {
        match self.0.dyn_render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
