use axum::{Extension, Form};
use axum::extract::State;
use axum::response::Redirect;
use sqlx::PgPool;
use gill_db::user::User;
use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::state::AppState;
use serde::Deserialize;
use crate::domain::ssh_key::RawSshkey;

#[derive(Deserialize, Debug)]
pub struct AddSshKeyForm {
    pub title: String,
    pub key: String,
}

pub async fn add(
    connected_user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Form(input): Form<AddSshKeyForm>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, connected_user).await else {
        return Err(AppError::Unauthorized);
    };
    let raw_key = RawSshkey::from(input.key);
    let (key_type, key) = raw_key.key_parts();
    user.add_ssh_key(&input.title, key, key_type, &db).await?;
    #[cfg(not(feature = "integration"))]
    gill_git::append_ssh_key(raw_key.full_key(), user.id).expect("Failed to append ssh key");
    Ok(Redirect::to("/settings/profile?tab=ssh-key"))
}

