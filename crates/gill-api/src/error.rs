use axum::response::{IntoResponse, Response};
use http::StatusCode;
use std::fmt::{write, Display, Formatter};

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl<T> From<T> for AppError
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        AppError(t.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}
