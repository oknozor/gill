use crate::{activities::follow::Follow, error::Error, instance::InstanceHandle};

use activitypub_federation::{
    core::{activity_queue::send_activity, object_id::ObjectId, signatures::PublicKey},
    data::Data,
    deser::context::WithContext,
    traits::{ActivityHandler, Actor, ApubObject},
    LocalInstance,
};
use activitystreams_kinds::actor::PersonType;
use gill_db::user::User;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApubUser(User);

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
    inbox: Url,
    public_key: PublicKey,
}

impl ApubUser {
    pub async fn followers(&self, instance: &InstanceHandle) -> Result<Vec<Url>, Error> {
        let db = instance.database();

        let followers = self.0.get_followers(i64::MAX, 0, db).await?;

        let followers = followers
            .into_iter()
            .map(|follower| follower.activity_pub_id)
            .filter_map(|url| Url::parse(&url).ok())
            .collect();

        Ok(followers)
    }

    pub fn followers_url(&self) -> Result<Url, Error> {
        Url::parse(&self.0.followers_url).map_err(Into::into)
    }

    fn activity_pub_id(&self) -> &str {
        &self.0.activity_pub_id
    }

    fn activity_pub_id_as_url(&self) -> Result<Url, Error> {
        Ok(Url::parse(self.activity_pub_id())?)
    }

    fn public_key(&self) -> Result<PublicKey, Error> {
        Ok(PublicKey::new_main_key(
            self.activity_pub_id_as_url()?,
            self.0.public_key.clone(),
        ))
    }

    pub async fn follow(&self, other: &ApubUser, instance: &InstanceHandle) -> Result<(), Error> {
        let follower = ObjectId::new(self.activity_pub_id_as_url()?);
        let following = ObjectId::new(other.activity_pub_id_as_url()?);
        let hostname = instance.local_instance().hostname();
        let activity_id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let activity_id = Url::parse(&activity_id)?;
        let follow = Follow::new(follower, following, activity_id);
        self.send(
            follow,
            vec![other.shared_inbox_or_inbox()],
            instance.local_instance(),
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn send<Activity>(
        &self,
        activity: Activity,
        recipients: Vec<Url>,
        local_instance: &LocalInstance,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Send + Sync,
        <Activity as ActivityHandler>::Error:
            From<anyhow::Error> + From<serde_json::Error> + From<Error>,
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

#[async_trait::async_trait]
impl ApubObject for ApubUser {
    type DataType = InstanceHandle;
    type ApubType = Person;
    type DbType = ApubUser;
    type Error = Error;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let db = data.database();
        let user = User::by_activity_pub_id(object_id.as_str(), db).await.ok();
        Ok(user.map(ApubUser))
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(Person {
            kind: Default::default(),
            id: ObjectId::new(self.activity_pub_id_as_url()?),
            inbox: self.inbox(),
            public_key: self.public_key()?,
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
        Ok(ApubUser(user))
    }
}

impl Actor for ApubUser {
    fn public_key(&self) -> &str {
        &self.0.public_key
    }
    fn inbox(&self) -> Url {
        self.activity_pub_id_as_url().expect("Invalid inbox url")
    }
}
