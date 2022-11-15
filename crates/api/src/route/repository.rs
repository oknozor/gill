use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::Deserialize;
use crate::error::AppError;

#[derive(Deserialize, JsonSchema)]
pub struct InitRepository {
    name: String,
}

pub async fn init_repository(
    axum::Json(repository): axum::Json<InitRepository>,
) -> Result<Response, AppError> {

    git_lib::init_bare(&repository.name)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
