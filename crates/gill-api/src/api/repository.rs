use crate::error::AppError;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::Json;
use gill_db::pagination::Pagination;
use gill_db::repository::{InitRepository, OwnedRepository, Repository};
use gill_db::user::User;
use sqlx::PgPool;

const PAGE_SIZE: i64 = 20;

pub async fn init(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<User>,
    Json(repository): Json<InitRepository>,
) -> Result<Response, AppError> {
    // TODO: handle database error
    Repository::create(user.id, &repository, &pool).await?;
    // #[cfg(not(feature = "integration"))]
    gill_git::repository::init_bare(&user.username, &repository.name)?;
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
