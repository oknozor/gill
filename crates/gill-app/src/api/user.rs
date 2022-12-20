use crate::error::AppError;
use activitypub_federation::core::signatures::generate_actor_keypair;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use gill_db::user::{CreateSSHKey, CreateUser, User};
use gill_settings::SETTINGS;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
}

pub async fn create(
    pool: Extension<PgPool>,
    Json(user): Json<CreateUserCommand>,
) -> Result<Response, AppError> {
    let keys = generate_actor_keypair()?;
    let scheme = if gill_settings::debug_mod() {
        "http://"
    } else {
        "https://"
    };
    let domain = &SETTINGS.domain;
    let username = user.username;
    let apub_id = format!("{scheme}{domain}/apub/users/{username}");
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

    User::create(user, &pool.0).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

struct RawSshkey {
    inner: String,
}

impl From<String> for RawSshkey {
    fn from(inner: String) -> Self {
        RawSshkey { inner }
    }
}

impl RawSshkey {
    fn key_parts(&self) -> (&str, &str) {
        let key = self.inner.trim();
        let mut parts = key.split(' ');
        let key_type = parts.next().expect("ssh key type");
        let key = parts.next().expect("ssh key");
        (key_type, key)
    }
}

pub async fn register_ssh_key(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(ssh_key): Json<CreateSSHKey>,
) -> Result<Response, AppError> {
    let key_name = ssh_key.name;
    let raw_key = RawSshkey::from(ssh_key.key);
    let (key_type, key) = raw_key.key_parts();
    User::add_ssh_key(user.id, &key_name, key, key_type, &pool).await?;
    #[cfg(not(feature = "integration"))]
    gill_git::append_ssh_key(&raw_key.inner, user.id).expect("Failed to append ssh key");
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
