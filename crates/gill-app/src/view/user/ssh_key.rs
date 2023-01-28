use crate::domain::user::ssh_key::RawSshkey;
use crate::error::{AppError, AppResult};
use crate::get_connected_user;
use crate::oauth::Oauth2User;

use axum::response::Redirect;
use axum::{Extension, Form};

use gill_authorize_derive::authorized;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize, Debug)]
pub struct AddSshKeyForm {
    pub title: String,
    pub key: String,
}

#[authorized]
pub async fn add(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Form(input): Form<AddSshKeyForm>,
) -> AppResult<Redirect> {
    let raw_key = RawSshkey::from(input.key);
    let (key_type, key) = raw_key.key_parts();
    user.add_ssh_key(&input.title, key, key_type, &db).await?;
    gill_git::ssh::append_key(raw_key.full_key(), user.id).expect("Failed to append ssh key");
    Ok(Redirect::to("/settings/profile?tab=ssh-key"))
}
