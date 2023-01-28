use crate::domain::id::ActivityPubId;
use crate::domain::repository::create::CreateRepository;
use crate::domain::user::User;
use crate::error::AppError;
use crate::error::AppResult;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::view::HtmlTemplate;
use activitypub_federation::core::signatures::generate_actor_keypair;
use askama::Template;
use axum::response::Redirect;
use axum::{Extension, Form};
use gill_authorize_derive::authorized;
use gill_settings::SETTINGS;
use serde::Deserialize;
use sqlx::PgPool;
use url::Url;

#[derive(Template)]
#[template(path = "repository/create.html")]
pub struct CreateRepositoryView {
    user: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateRepositoryForm {
    pub name: String,
    pub summary: Option<String>,
}

#[authorized]
pub async fn submit(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Form(input): Form<CreateRepositoryForm>,
) -> AppResult<Redirect> {
    let create_repository_command = input.map_to_domain(&user)?;
    let repository = create_repository_command.save(&db).await?;
    gill_git::init::init_bare(&user.username, &repository.name)?;

    Ok(Redirect::to(&format!(
        "/{}/{}",
        user.username, repository.name
    )))
}

#[authorized]
pub async fn view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
) -> AppResult<HtmlTemplate<CreateRepositoryView>> {
    Ok(HtmlTemplate(CreateRepositoryView {
        user: Some(user.username),
    }))
}

impl CreateRepositoryForm {
    fn map_to_domain(self, user: &User) -> AppResult<CreateRepository> {
        let protocol = &SETTINGS.protocol();
        let user_name = user.username.clone();
        let domain = &SETTINGS.domain;
        let apub_id = format!(
            "{protocol}://{domain}/users/{user_name}/repositories/{}",
            self.name
        );

        let ssh_port = SETTINGS.ssh_port;
        // TODO: domain and port should be split split in config to avoid this
        //  or we should build this from Url
        let clone_uri = if let Some((domain, _port)) = domain.rsplit_once(':') {
            format!(
                "ssh://git@{domain}:{ssh_port}/~/{user_name}/{}.git",
                self.name
            )
        } else {
            format!(
                "ssh://git@{domain}:{ssh_port}/~/{user_name}/{}.git",
                self.name
            )
        };

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
            domain: domain.to_string(),
            is_local: true,
        })
    }
}