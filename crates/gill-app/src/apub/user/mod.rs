use crate::apub::repository::star::Star;
use crate::apub::repository::watch::Watch;
use crate::apub::repository::RepositoryWrapper;
use crate::apub::ticket::create::CreateTicket;
use crate::apub::GillApubObject;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::core::signatures::PublicKey;
use activitypub_federation::data::Data;
use activitypub_federation::traits::{ActivityHandler, Actor, ApubObject};
use activitystreams_kinds::actor::PersonType;
use async_session::async_trait;
use follow::Follow;
use gill_db::user::{CreateUser, User};
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod follow;

#[derive(Debug, Clone)]
pub struct UserWrapper(User);

impl From<User> for UserWrapper {
    fn from(user: User) -> Self {
        UserWrapper(user)
    }
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    Follow(Follow),
    CreateIssue(CreateTicket),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubUser {
    #[serde(rename = "type")]
    pub kind: PersonType,
    pub id: ObjectId<UserWrapper>,
    pub email: Option<String>,
    pub username: String,
    pub outbox: Url,
    pub inbox: Url,
    pub domain: String,
    pub followers: Url,
    pub public_key: PublicKey,
}

#[async_trait]
impl GillApubObject for UserWrapper {
    fn view_uri(&self) -> String {
        format!("/{}", self.0.username)
    }

    fn followers_url(&self) -> Result<Url, AppError> {
        Url::parse(&self.0.followers_url).map_err(Into::into)
    }

    async fn followers(&self, instance: &InstanceHandle) -> Result<Vec<Url>, AppError> {
        let db = instance.database();
        let followers = self.0.get_followers(i64::MAX, 0, db).await?;

        let followers = followers
            .into_iter()
            .map(|follower| follower.inbox_url)
            .filter_map(|url| Url::parse(&url).ok())
            .collect();

        Ok(followers)
    }

    fn local_id(&self) -> i32 {
        self.0.id
    }

    fn activity_pub_id(&self) -> &str {
        &self.0.activity_pub_id
    }

    fn public_key_with_owner(&self) -> Result<PublicKey, ParseError> {
        Ok(PublicKey::new_main_key(
            self.activity_pub_id_as_url()?,
            self.0.public_key.clone(),
        ))
    }

    fn private_key(&self) -> Option<String> {
        self.0.private_key.clone()
    }
}

impl UserWrapper {
    pub async fn add_follower(
        &self,
        follower_id: i32,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let db = instance.database();
        self.0.add_follower(follower_id, db).await?;
        Ok(())
    }

    pub fn activity_pub_id_from_namespace(user: &str) -> anyhow::Result<ObjectId<Self>> {
        let domain = &SETTINGS.domain;
        let scheme = if SETTINGS.debug { "http" } else { "https" };
        let url = Url::from_str(&format!("{scheme}://{domain}/apub/users/{user}"))?;
        Ok(ObjectId::new(url))
    }

    pub async fn follow(
        &self,
        other: &UserWrapper,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let follower = ObjectId::new(self.activity_pub_id_as_url()?);
        let following = ObjectId::new(other.activity_pub_id_as_url()?);
        let hostname = instance.local_instance().hostname();
        let activity_id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let activity_id = Url::parse(&activity_id)?;
        let follow = Follow::new(follower, following, activity_id);
        tracing::debug!(
            "Sending follow activity to user inboc {}",
            other.shared_inbox_or_inbox()
        );
        self.send(
            follow,
            vec![other.shared_inbox_or_inbox()],
            instance.local_instance(),
        )
        .await?;
        Ok(())
    }

    pub async fn watch_repository(
        &self,
        other: &RepositoryWrapper,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let watcher = ObjectId::new(self.activity_pub_id_as_url()?);
        let watching = ObjectId::new(other.activity_pub_id_as_url()?);
        let hostname = instance.local_instance().hostname();
        let activity_id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let activity_id = Url::parse(&activity_id)?;
        let watch = Watch::new(watcher, watching, activity_id);

        tracing::debug!(
            "Sending watch activity to repository inbox {}",
            other.shared_inbox_or_inbox()
        );

        self.send(
            watch,
            vec![other.shared_inbox_or_inbox()],
            instance.local_instance(),
        )
        .await?;
        Ok(())
    }

    pub async fn star_repository(
        &self,
        other: &RepositoryWrapper,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let starred_by = ObjectId::new(self.activity_pub_id_as_url()?);
        let starred = ObjectId::new(other.activity_pub_id_as_url()?);
        let hostname = instance.local_instance().hostname();
        let activity_id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let activity_id = Url::parse(&activity_id)?;
        let star = Star::new(starred_by, starred, activity_id);
        tracing::debug!(
            "Sending star activity to repository inbox {}",
            other.shared_inbox_or_inbox()
        );

        self.send(
            star,
            vec![other.shared_inbox_or_inbox()],
            instance.local_instance(),
        )
        .await?;
        Ok(())
    }
}

#[async_trait]
impl ApubObject for UserWrapper {
    type DataType = InstanceHandle;
    type ApubType = ApubUser;
    type DbType = UserWrapper;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let db = data.database();
        let user = User::by_activity_pub_id(object_id.as_str(), db).await?;
        Ok(user.map(UserWrapper))
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(ApubUser {
            kind: Default::default(),
            id: ObjectId::new(self.activity_pub_id_as_url()?),
            public_key: self.public_key_with_owner()?,
            email: self.0.email,
            username: self.0.username,
            outbox: Url::parse(&self.0.outbox_url)?,
            domain: self.0.domain,
            inbox: Url::parse(&self.0.inbox_url)?,
            followers: Url::parse(&self.0.followers_url)?,
        })
    }

    async fn verify(
        _apub: &Self::ApubType,
        _expected_domain: &Url,
        _data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn from_apub(
        apub: Self::ApubType,
        data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<Self, Self::Error> {
        let db = data.database();
        let id = Url::from(apub.id);
        let user = User::by_activity_pub_id(id.as_str(), db).await?;
        if let Some(user) = user {
            Ok(UserWrapper(user))
        } else {
            let user = CreateUser {
                username: apub.username,
                email: apub.email,
                private_key: None,
                public_key: apub.public_key.public_key_pem,
                activity_pub_id: id.to_string(),
                outbox_url: apub.outbox.to_string(),
                inbox_url: apub.inbox.to_string(),
                domain: apub.domain,
                followers_url: apub.followers.to_string(),
                is_local: false,
            };

            let user = User::create(user, db).await?;
            Ok(UserWrapper(user))
        }
    }
}

impl Actor for UserWrapper {
    fn public_key(&self) -> &str {
        &self.0.public_key
    }
    fn inbox(&self) -> Url {
        Url::from_str(&self.0.inbox_url).expect("Invalid inbox url")
    }
}
