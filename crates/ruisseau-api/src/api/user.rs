use crate::error::AppError;
use crate::SETTINGS;
use activitypub_federation::core::signatures::generate_actor_keypair;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use ruisseau_db::user::{CreateSSHKey, CreateUser, User};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize, Serialize)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
}

pub async fn create(
    pool: Extension<PgPool>,
    Json(user): Json<CreateUserDto>,
) -> Result<Response, AppError> {
    let keys = generate_actor_keypair()?;
    println!("{:?}", SETTINGS);
    let user = CreateUser {
        username: user.username.clone(),
        email: user.email,
        private_key: Some(keys.private_key),
        public_key: keys.public_key,
        followers_url: format!("http://{}/{}/followers/", SETTINGS.domain, user.username),
        outbox_url: format!("http://{}/{}/outbox/", SETTINGS.domain, user.username),
        inbox_url: format!("http://{}/{}/inbox/", SETTINGS.domain, user.username),
        activity_pub_id: format!("http://{}/{}/", SETTINGS.domain, user.username),
        domain: SETTINGS.domain.clone(),
        is_local: true,
    };

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
