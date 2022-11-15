use crate::error::AppError;
use aide::axum::IntoApiResponse;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateSSHKey {
    user_id: i64,
    key: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateUser {
    username: String,
}

#[derive(Deserialize, Serialize, JsonSchema, PartialEq, Eq, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[debug_handler]
pub async fn create(
    pool: Extension<PgPool>,
    Json(user): Json<CreateUser>,
) -> Result<Response, AppError> {
    let username = user.username;
    sqlx::query!(
        // language=PostgreSQL
        r#"
            insert into "users"(username)
            values ($1)
        "#,
        username
    )
    .execute(&*pool)
    .await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

use axum_macros::debug_handler;

#[debug_handler]
pub async fn by_id(
    pool: Extension<PgPool>,
    Path(user_id): Path<i32>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        // language=PostgreSQL
        r#"
            select *
            from users
            where id = $1
        "#,
        user_id
    )
    .fetch_one(&*pool)
    .await?;

    Ok(Json(user))
}

pub async fn register_ssh_key(Json(ssh_key): Json<CreateSSHKey>) -> impl IntoApiResponse {
    git_lib::append_ssh_key(&ssh_key.key).expect("Failed to append ssh key");

    "Ok".to_string()
}
