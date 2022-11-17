use crate::error::AppError;
use crate::route::user::User;
use crate::route::Pagination;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

const PAGE_SIZE: i64 = 20;

#[derive(Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Debug)]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub owner_id: i32,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Debug)]
pub struct OwnedRepository {
    pub id: i32,
    pub owner_id: i32,
    pub name: String,
    pub owner_name: String,
}

impl OwnedRepository {
    pub fn full_path(&self) -> String {
        format!("{}/{}", self.owner_name, self.name)
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct InitRepository {
    pub name: String,
}

pub async fn init(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<User>,
    Json(repository): Json<InitRepository>,
) -> Result<Response, AppError> {
    // TODO: handle database error
    Repository::create(user.id, &repository, &pool).await?;
    #[cfg(not(feature = "integration"))]
    ruisseau_git::repository::init_bare(&user.username, &repository.name)?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub async fn list(
    Extension(pool): Extension<PgPool>,
    pagination: Option<Query<Pagination>>,
) -> Result<Json<Vec<OwnedRepository>>, AppError> {
    let pagination = pagination.unwrap_or_default();
    let offset = (pagination.page.get() - 1) * PAGE_SIZE;
    Repository::list(PAGE_SIZE, offset, &pool)
        .await
        .map(Json)
        .map_err(AppError::from)
}
