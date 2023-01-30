use crate::error::AppResult;

use activitypub_federation::core::signatures::generate_actor_keypair;

use crate::domain::id::ActivityPubId;
use crate::domain::repository::create::CreateRepository;
use crate::domain::user::User;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::Json;
use gill_settings::SETTINGS;
use serde::Deserialize;
use sqlx::PgPool;

use url::Url;

#[derive(Deserialize)]
pub struct CreateRepositoryCommand {
    pub name: String,
    pub summary: Option<String>,
}

impl CreateRepositoryCommand {
    fn map_to_domain(self, user: &User) -> AppResult<CreateRepository> {
        let apub_id = self.generate_activity_pub_id(user);
        let clone_uri = self.generate_clone_uri(user);
        let key_pair = generate_actor_keypair()?;
        let activity_pub_id = ActivityPubId::try_from(apub_id.clone())?;

        // Note that for now 'ticket_tracked_by' and 'send_patches_to' are
        // the local repository owner by default. We might want to change this later
        Ok(CreateRepository {
            activity_pub_id: activity_pub_id.clone(),
            name: self.name,
            summary: self.summary,
            private: false,
            inbox_url: Url::parse(&format!("{apub_id}/inbox"))?,
            outbox_url: Url::parse(&format!("{apub_id}/outbox"))?,
            followers_url: Url::parse(&format!("{apub_id}/followers"))?,
            attributed_to: user.activity_pub_id.clone(),
            clone_uri: Url::parse(&clone_uri)?,
            public_key: key_pair.public_key,
            private_key: Some(key_pair.private_key),
            ticket_tracked_by: activity_pub_id.clone(),
            send_patches_to: activity_pub_id,
            domain: SETTINGS.domain.to_string(),
            is_local: true,
        })
    }

    fn generate_clone_uri(&self, user: &User) -> String {
        let ssh_port = SETTINGS.ssh_port;
        let domain = SETTINGS.domain_url().expect("valid domain");
        let host = domain.host_str().expect("valid host");
        let repository_name = &self.name;
        let username = &user.username;

        if ssh_port == 22 {
            format!("git@{host}:{username}/{repository_name}.git")
        } else {
            format!("ssh://git@{host}:{ssh_port}/~/{username}/{repository_name}.git")
        }
    }

    fn generate_activity_pub_id(&self, user: &User) -> String {
        let protocol = &SETTINGS.protocol();
        let user_name = user.username.clone();
        let domain = &SETTINGS.domain;
        format!(
            "{protocol}://{domain}/users/{user_name}/repositories/{}",
            self.name
        )
    }
}

pub async fn init(
    Extension(db): Extension<PgPool>,
    Extension(user): Extension<User>,
    Json(repository): Json<CreateRepositoryCommand>,
) -> AppResult<Response> {
    let create_repository_command = repository.map_to_domain(&user)?;
    let repository = create_repository_command.save(&db).await?;
    gill_git::init::init_bare(&user.username, &repository.name)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
