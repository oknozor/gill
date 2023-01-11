use crate::get_connected_user;
use crate::oauth::Oauth2User;
use activitypub_federation::core::object_id::ObjectId;

use crate::apub::common::GillApubObject;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use anyhow::anyhow;
use axum::extract::State;
use axum::response::Redirect;
use axum::{Extension, Form};
use serde::Deserialize;
use sqlx::PgPool;
use url::Url;
use webfinger::Webfinger;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct FollowForm {
    pub follow: String,
}

//TODO :  We need to refactor this big pile of mud and think about how to handle this
// properly
pub async fn follow_form(
    State(data): State<AppState>,
    connected_user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Form(input): Form<FollowForm>,
) -> Result<Redirect, AppError> {
    let Some(_user) = get_connected_user(&db, connected_user).await else {
        return Err(AppError::from(anyhow!("Unauthorized")));
    };

    // First attempt to parse the whole user url, then fallback to webfinger
    if let Ok(url) = Url::parse(&input.follow) {
        // FIXME: this is not compatible with other forgefed instances and should not be done this way
        //  anyway, we should discuss this with forgefed people to find a common ground here
        if input.follow.contains('/') {
            let repository = ObjectId::<Repository>::new(url);
            let repository = repository
                .dereference(&data.instance, data.instance.local_instance(), &mut 0)
                .await?;

            let user = repository.owner_apub_id()?;
            user.dereference(&data.instance, data.instance.local_instance(), &mut 0)
                .await?;

            Ok(Redirect::to(&repository.view_uri()))
        } else {
            let user = ObjectId::<User>::new(url);
            let user = user
                .dereference(&data.instance, data.instance.local_instance(), &mut 0)
                .await?;

            Ok(Redirect::to(&user.view_uri()))
        }
    } else if let Ok(webfinger) = resolve_webfinger(&input.follow).await {
        let page_link = webfinger
            .links
            .iter()
            .find(|link| link.rel == "repository-page");

        let apub_link = webfinger.links.iter().find(|link| link.rel == "self");

        if let (Some(apub_link), Some(page_link)) = (apub_link, page_link) {
            let url = Url::parse(apub_link.href.as_ref().unwrap())?;
            let repository = ObjectId::<Repository>::new(url);

            let repository = repository
                .dereference(&data.instance, data.instance.local_instance(), &mut 0)
                .await?;
            println!("{repository:?}");
            let user = repository.owner_apub_id()?;
            user.dereference(&data.instance, data.instance.local_instance(), &mut 0)
                .await?;

            repository
                .owner_apub_id()?
                .dereference(&data.instance, data.instance.local_instance(), &mut 0)
                .await?;

            Ok(Redirect::to(page_link.href.as_ref().unwrap()))
        } else {
            let page_link = webfinger
                .links
                .iter()
                .find(|link| link.rel == "user-profile");

            let apub_link = webfinger.links.iter().find(|link| link.rel == "self");

            if let (Some(apub_link), Some(page_link)) = (apub_link, page_link) {
                let url = Url::parse(apub_link.href.as_ref().unwrap())?;
                let user = ObjectId::<User>::new(url);
                user.dereference(&data.instance, data.instance.local_instance(), &mut 0)
                    .await?;

                Ok(Redirect::to(page_link.href.as_ref().unwrap()))
            } else {
                Err(AppError::from(anyhow!(
                    "Bad response from webfinger endpoind"
                )))
            }
        }
    } else {
        Err(AppError::from(anyhow!("Invalid user identifier")))
    }
}

async fn resolve_webfinger(webfinger: &str) -> anyhow::Result<Webfinger> {
    let webfinger = webfinger::resolve(webfinger, false)
        .await
        .expect("Web finger error");
    Ok(webfinger)
}
