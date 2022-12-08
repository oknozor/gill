use axum::response::{IntoResponse, Response};
use http::StatusCode;

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

impl AppError {
    pub fn not_found(self) -> Response {
        (StatusCode::NOT_FOUND, format!("{}", self.0)).into_response()
    }
}
