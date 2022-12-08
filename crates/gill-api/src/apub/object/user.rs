use crate::apub::activities::follow::Follow;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::{
    core::{activity_queue::send_activity, object_id::ObjectId, signatures::PublicKey},
    data::Data,
    deser::context::WithContext,
    traits::{ActivityHandler, Actor, ApubObject},
    LocalInstance,
};
use activitystreams_kinds::actor::PersonType;
use std::str::FromStr;

use axum::async_trait;
use gill_db::user::{CreateUser, User};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApubUser(User);

impl From<User> for ApubUser {
    fn from(user: User) -> Self {
        ApubUser(user)
    }
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    Follow(Follow),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    kind: PersonType,
    id: ObjectId<ApubUser>,
    email: Option<String>,
    username: String,
    outbox: Url,
    inbox: Url,
    domain: String,
    followers: Url,
    public_key: PublicKey,
}

impl ApubUser {
    pub async fn followers(&self, instance: &InstanceHandle) -> Result<Vec<Url>, AppError> {
        let db = instance.database();
        let followers = self.0.get_followers(i64::MAX, 0, db).await?;

        let followers = followers
            .into_iter()
            .map(|follower| follower.activity_pub_id)
            .filter_map(|url| Url::parse(&url).ok())
            .collect();

        Ok(followers)
    }

    pub async fn add_follower(
        &self,
        follower_id: i32,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let db = instance.database();
        self.0.add_follower(follower_id, db).await?;
        Ok(())
    }

    pub fn followers_url(&self) -> Result<Url, AppError> {
        Url::parse(&self.0.followers_url).map_err(Into::into)
    }

    fn activity_pub_id(&self) -> &str {
        &self.0.activity_pub_id
    }

    pub fn local_id(&self) -> i32 {
        self.0.id
    }

    fn activity_pub_id_as_url(&self) -> Result<Url, AppError> {
        Ok(Url::parse(self.activity_pub_id())?)
    }

    fn public_key(&self) -> Result<PublicKey, AppError> {
        Ok(PublicKey::new_main_key(
            self.activity_pub_id_as_url()?,
            self.0.public_key.clone(),
        ))
    }

    pub async fn follow(
        &self,
        other: &ApubUser,
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

    pub async fn send<Activity>(
        &self,
        activity: Activity,
        recipients: Vec<Url>,
        local_instance: &LocalInstance,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Send + Sync,
        <Activity as ActivityHandler>::Error:
            From<anyhow::Error> + From<serde_json::Error> + From<AppError>,
    {
        let activity = WithContext::new_default(activity);
        send_activity(
            activity,
            self.public_key()?,
            self.0.private_key.clone().expect("has private key"),
            recipients,
            local_instance,
        )
        .await?;
        Ok(())
    }
}

#[async_trait]
impl ApubObject for ApubUser {
    type DataType = InstanceHandle;
    type ApubType = Person;
    type DbType = ApubUser;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let db = data.database();
        let user = User::by_activity_pub_id(object_id.as_str(), db).await?;
        Ok(user.map(ApubUser))
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(Person {
            kind: Default::default(),
            id: ObjectId::new(self.activity_pub_id_as_url()?),
            public_key: self.public_key()?,
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
            Ok(ApubUser(user))
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
            Ok(ApubUser(user))
        }
    }
}

impl Actor for ApubUser {
    fn public_key(&self) -> &str {
        &self.0.public_key
    }
    fn inbox(&self) -> Url {
        Url::from_str(&self.0.inbox_url).expect("Invalid inbox url")
    }
}
