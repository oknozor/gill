use crate::error::AppError;

use activitypub_federation::core::signatures::generate_actor_keypair;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::Json;
use gill_db::repository::create::CreateRepository;
use gill_db::repository::Repository;
use gill_db::user::User;
use gill_settings::SETTINGS;
use serde::Deserialize;
use sqlx::PgPool;
use std::io;

#[derive(Deserialize)]
pub struct CreateRepositoryCommand {
    pub name: String,
    pub summary: Option<String>,
}

impl CreateRepositoryCommand {
    fn map_to_db(self, user: &User) -> Result<CreateRepository, io::Error> {
        let scheme = if gill_settings::debug_mod() {
            "http://"
        } else {
            "https://"
        };
        let user_name = user.username.clone();
        let domain = &SETTINGS.domain;
        let apub_id = format!(
            "{scheme}{domain}/apub/users/{user_name}/repositories/{}",
            self.name
        );
        let clone_uri = format!("git@{domain}:{user_name}/{}.git", self.name);
        let key_pair = generate_actor_keypair()?;

        // Note that for now 'ticket_tracked_by' and 'send_patches_to' are
        // the local repository owner by default. We might want to change this later
        Ok(CreateRepository {
            activity_pub_id: apub_id.clone(),
            name: self.name,
            summary: self.summary,
            private: false,
            inbox_url: format!("{apub_id}/inbox"),
            outbox_url: format!("{apub_id}/outbox"),
            followers_url: format!("{apub_id}/followers"),
            attributed_to: user.activity_pub_id.clone(),
            clone_uri,
            public_key: key_pair.public_key,
            private_key: Some(key_pair.private_key),
            ticket_tracked_by: user.activity_pub_id.clone(),
            send_patches_to: user.activity_pub_id.clone(),
            domain: domain.to_string(),
            is_local: true,
        })
    }
}

pub async fn init(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<User>,
    Json(repository): Json<CreateRepositoryCommand>,
) -> Result<Response, AppError> {
    // TODO: handle database error
    let create_repository_command = repository.map_to_db(&user)?;
    let repository = Repository::create(&create_repository_command, &pool).await?;
    // #[cfg(not(feature = "integration"))]
    gill_git::repository::init::init_bare(&user.username, &repository.name)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
