use axum::response::{IntoResponse, Response};
use http::StatusCode;
use tracing::error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Internal(anyhow::Error),
    Unauthorized,
    NotFound,
}

impl<T> From<T> for AppError
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        let err = t.into();
        error!("{err}");
        AppError::Internal(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{error}")).into_response()
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED").into_response(),
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND").into_response(),
        }
    }
}
