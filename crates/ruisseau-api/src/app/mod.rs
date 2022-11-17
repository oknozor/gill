use askama::Template;
use axum::response::Response;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

struct HtmlTemplate<T>(T);


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

pub mod repositories_pages;