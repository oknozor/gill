use crate::error::AppError;
use crate::route::user::User;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Debug)]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub owner_id: i32,
}

#[derive(Deserialize, JsonSchema)]
pub struct InitRepository {
    pub name: String,
}

pub async fn init_repository(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(repository): Json<InitRepository>,
) -> Result<Response, AppError> {
    // TODO: handle database error
    Repository::create(user.id, &repository, &pool).await?;
    #[cfg(not(feature = "integration"))]
    ruisseau_git::init_bare(&user.username, &repository.name)?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
