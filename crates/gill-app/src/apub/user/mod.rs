use crate::apub::common::GillApubObject;

use crate::apub::ticket::accept::AcceptTicket;
use crate::apub::ticket::comment::create::CreateTicketComment;

use crate::domain::id::ActivityPubId;
use crate::domain::user::create::CreateUser;
use crate::domain::user::User;
use crate::error::{AppError, AppResult};
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::core::signatures::PublicKey;
use activitypub_federation::data::Data;
use activitypub_federation::traits::{ActivityHandler, Actor, ApubObject};
use activitystreams_kinds::actor::PersonType;
use async_session::async_trait;
use follow::Follow;

use serde::{Deserialize, Serialize};

use url::{ParseError, Url};

pub mod follow;

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    Follow(Follow),
    AcceptTicket(AcceptTicket),
    CreateIssueComment(CreateTicketComment),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubUser {
    #[serde(rename = "type")]
    pub kind: PersonType,
    pub id: ObjectId<User>,
    pub email: Option<String>,
    pub username: String,
    pub outbox: Url,
    pub inbox: Url,
    pub domain: String,
    pub followers: Url,
    pub public_key: PublicKey,
}

#[async_trait]
impl GillApubObject for User {
    fn view_uri(&self) -> String {
        format!("/{}", self.username)
    }

    async fn followers(&self, instance: &InstanceHandle) -> AppResult<Vec<Url>> {
        let db = instance.database();
        let followers = self.get_followers(i64::MAX, 0, db).await?;

        let followers = followers
            .into_iter()
            .map(|follower| follower.inbox_url)
            .collect();

        Ok(followers)
    }

    fn local_id(&self) -> i32 {
        self.id
    }

    fn public_key_with_owner(&self) -> Result<PublicKey, ParseError> {
        Ok(PublicKey::new_main_key(
            self.activity_pub_id.clone().into(),
            self.public_key.clone(),
        ))
    }

    fn private_key(&self) -> Option<String> {
        self.private_key.clone()
    }
}

impl User {}

#[async_trait]
impl ApubObject for User {
    type DataType = InstanceHandle;
    type ApubType = ApubUser;
    type DbType = User;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let db = data.database();
        User::by_activity_pub_id_optional(object_id.as_str(), db).await
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        let public_key = self.public_key_with_owner()?;
        Ok(ApubUser {
            kind: Default::default(),
            id: self.activity_pub_id.into(),
            public_key,
            email: self.email,
            username: self.username,
            outbox: self.outbox_url,
            domain: self.domain,
            inbox: self.inbox_url,
            followers: self.followers_url,
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
        let user = User::by_activity_pub_id_optional(id.as_str(), db).await?;
        if let Some(user) = user {
            Ok(user)
        } else {
            let user = CreateUser {
                username: apub.username,
                email: apub.email,
                private_key: None,
                public_key: apub.public_key.public_key_pem,
                activity_pub_id: ActivityPubId::from(id),
                outbox_url: apub.outbox,
                inbox_url: apub.inbox,
                domain: apub.domain,
                followers_url: apub.followers,
                is_local: false,
            };

            user.save(db).await
        }
    }
}

impl Actor for User {
    fn public_key(&self) -> &str {
        &self.public_key
    }

    fn inbox(&self) -> Url {
        self.inbox_url.clone()
    }
}
