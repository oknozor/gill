use crate::error::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use ruisseau_db::user::{CreateSSHKey, CreateUser, User};
use sqlx::PgPool;

pub async fn create(
    pool: Extension<PgPool>,
    Json(user): Json<CreateUser>,
) -> Result<Response, AppError> {
    User::create(user, &pool.0).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub async fn register_ssh_key(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(ssh_key): Json<CreateSSHKey>,
) -> Result<Response, AppError> {
    User::add_ssh_key(user.id, &ssh_key.key, &pool).await?;
    #[cfg(not(feature = "integration"))]
    ruisseau_git::append_ssh_key(&ssh_key.key).expect("Failed to append ssh key");
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
