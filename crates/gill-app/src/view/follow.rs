use crate::apub::object::user::UserWrapper;

use crate::oauth::Oauth2User;
use crate::view::get_connected_user;
use activitypub_federation::core::object_id::ObjectId;

use anyhow::anyhow;
use axum::extract::State;
use axum::{Extension, Form};
use serde::Deserialize;
use sqlx::PgPool;
use url::Url;

#[derive(Deserialize, Debug)]
pub struct FollowForm {
    pub follow: String,
}

use crate::error::AppError;
use crate::state::AppState;

pub async fn follow_form(
    State(data): State<AppState>,
    connected_user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Form(input): Form<FollowForm>,
) -> Result<(), AppError> {
    let Some(user) = get_connected_user(&db, connected_user).await else {
        return Err(AppError::from(anyhow!("Unauthorized")))
    };

    // First attempt to parse the whole user url, then fallback to webfinger
    let url = if let Ok(url) = Url::parse(&input.follow) {
        url
    } else if let Ok(Some(url)) = resolve_webfinger(&input.follow).await {
        url
    } else {
        return Err(AppError::from(anyhow!("Invalid user identifier")));
    };

    let user_to_follow = ObjectId::<UserWrapper>::new(url);
    let user_to_follow = user_to_follow
        .dereference(&data.instance, data.instance.local_instance(), &mut 0)
        .await?;

    UserWrapper::from(user)
        .follow(&user_to_follow, &data.instance)
        .await?;

    Ok(())
}

async fn resolve_webfinger(webfinger: &str) -> anyhow::Result<Option<Url>> {
    let webfinger = webfinger::resolve(webfinger, false)
        .await
        .expect("Web finger error");
    Ok(webfinger
        .links
        .iter()
        .find(|link| link.rel == "self")
        .and_then(|link| link.href.as_ref())
        .and_then(|url| Url::parse(url).ok()))
}
