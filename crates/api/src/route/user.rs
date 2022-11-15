use serde::{Deserialize, Serialize};
use aide::axum::IntoApiResponse;
use axum::Json;
use axum::extract::State;
use schemars::JsonSchema;
use sqlx::PgPool;
use crate::error::AppError;
use axum_macros::debug_handler;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateSSHKey {
    key: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateUser {
    username: String,
}

#[debug_handler]
pub async fn create(
    State(pool): State<PgPool>,
    Json(user): Json<CreateUser>,
) -> Result<(), AppError> {
    let username = user.username;
    sqlx::query!(
       r#"
            insert into "users"(username)
            values ($1)
        "#,
        username
    )
        .execute(&pool)
        .await?;

    Ok(())
}

pub async fn register_ssh_key(Json(ssh_key): Json<CreateSSHKey>) -> impl IntoApiResponse {
    println!("Append ssh key {}", ssh_key.key);
    git_lib::append_ssh_key(&ssh_key.key)
        .expect("Failed to append ssh key");

    "Ok".to_string()
}
