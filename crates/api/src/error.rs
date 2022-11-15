use aide::OperationOutput;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct AppError(pub eyre::Error);

impl OperationOutput for AppError {
    type Inner = String;
}

// TODO: thiserror
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
