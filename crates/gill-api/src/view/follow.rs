use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::data::Data;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::{InstanceSettings, LocalInstance};
use activitypub_federation::traits::ApubObject;
use axum::{Extension, Form};
use axum::extract::State;
use sqlx::PgPool;
use crate::apub::activities::follow::Follow;
use crate::apub::object::user::{ApubUser, Person};
use crate::oauth::{AppState, Oauth2User};
use crate::view::{get_connected_user, get_connected_user_username};
use serde::{Deserialize};
use url::Url;
use crate::instance::{Instance, InstanceHandle};

#[derive(Deserialize, Debug)]
pub struct FollowForm {
    pub follow: String,
}

use axum_macros::debug_handler;
use reqwest::Client;
use gill_db::user::User;
use crate::error::AppError;

pub async fn follow_form(
    State(data): State<AppState>,
    connected_user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Form(input): Form<FollowForm>,
) -> Result<(), AppError>{
    let user = get_connected_user(&db, connected_user).await;
    if let Some(user) = user {
        let url = Url::parse(&input.follow).unwrap();
        let instance_host = url.host_str().unwrap();
        let instance_host = instance_host.to_string();
        let user_to_follow = ObjectId::<ApubUser>::new(url);
        let settings = InstanceSettings::builder()
        .debug(true)
        .build()?;

        let local_instance = LocalInstance::new(instance_host, Client::default().into(), settings);
        let user_to_follow= user_to_follow.dereference(&data.instance, &local_instance, &mut 0)
            .await?;
    }

    Ok(())
}