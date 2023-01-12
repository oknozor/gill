use crate::domain::id::ActivityPubId;
use crate::domain::user::create::CreateUser;
use crate::domain::user::ssh_key::CreateSSHKey;
use crate::domain::user::ssh_key::RawSshkey;
use crate::domain::user::User;
use crate::error::{AppResult};
use activitypub_federation::core::signatures::generate_actor_keypair;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum_macros::debug_handler;
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use url::Url;

#[derive(Deserialize)]
pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
}

pub async fn create(
    Extension(db): Extension<PgPool>,
    Json(user): Json<CreateUserCommand>,
) -> AppResult<Response> {
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
        followers_url: Url::parse(&format!("{apub_id}/followers"))?,
        outbox_url: Url::parse(&format!("{apub_id}/outbox"))?,
        inbox_url: Url::parse(&format!("{apub_id}/inbox"))?,
        activity_pub_id: ActivityPubId::try_from(apub_id)?,
        domain: SETTINGS.domain.clone(),
        is_local: true,
    };

    user.save(&db).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Serialize, Deserialize)]
pub struct CreateSSHKeyDto {
    pub name: String,
    pub key: String,
}

impl From<CreateSSHKeyDto> for CreateSSHKey {
    fn from(val: CreateSSHKeyDto) -> Self {
        CreateSSHKey {
            name: val.name,
            key: val.key,
        }
    }
}

#[debug_handler]
pub async fn register_ssh_key(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(ssh_key): Json<CreateSSHKeyDto>,
) -> AppResult<Response> {
    let key_name = ssh_key.name;
    let raw_key = RawSshkey::from(ssh_key.key);
    let (key_type, key) = raw_key.key_parts();
    user.add_ssh_key(&key_name, key, key_type, &pool).await?;
    gill_git::append_ssh_key(raw_key.full_key(), user.id).expect("Failed to append ssh key");
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
