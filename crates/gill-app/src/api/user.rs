use crate::domain::ssh_key::RawSshkey;
use crate::error::AppError;
use activitypub_federation::core::signatures::generate_actor_keypair;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use gill_db::user::{CreateSSHKey, CreateUser, User};
use gill_db::Insert;
use gill_settings::SETTINGS;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
}

pub async fn create(
    Extension(db): Extension<PgPool>,
    Json(user): Json<CreateUserCommand>,
) -> Result<Response, AppError> {
    let keys = generate_actor_keypair()?;
    let protocol = SETTINGS.protocol();
    let domain = &SETTINGS.domain;
    let username = user.username;
    let apub_id = format!("{protocol}://{domain}/apub/users/{username}");
    let user = CreateUser {
        username: username.clone(),
        email: Some(user.email),
        private_key: Some(keys.private_key),
        public_key: keys.public_key,
        followers_url: format!("{apub_id}/followers"),
        outbox_url: format!("{apub_id}/outbox"),
        inbox_url: format!("{apub_id}/inbox"),
        activity_pub_id: apub_id,
        domain: SETTINGS.domain.clone(),
        is_local: true,
    };

    user.insert(&db).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub async fn register_ssh_key(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(ssh_key): Json<CreateSSHKey>,
) -> Result<Response, AppError> {
    let key_name = ssh_key.name;
    let raw_key = RawSshkey::from(ssh_key.key);
    let (key_type, key) = raw_key.key_parts();
    user.add_ssh_key(&key_name, key, key_type, &pool).await?;
    #[cfg(not(feature = "integration"))]
    gill_git::append_ssh_key(raw_key.full_key(), user.id).expect("Failed to append ssh key");
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
