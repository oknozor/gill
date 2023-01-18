use crate::apub::common::GillApubObject;
use crate::error::{AppError, AppResult};
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::core::signatures::PublicKey;
use activitypub_federation::data::Data;
use activitypub_federation::traits::{ActivityHandler, Actor, ApubObject};
use activitystreams_kinds::kind;
use async_session::async_trait;
use fork::Fork;
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};

use crate::apub::ticket::comment::create::CreateTicketComment;
use crate::apub::ticket::offer::OfferTicket;
use crate::domain::repository::create::CreateRepository;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use star::Star;
use std::str::FromStr;
use url::{ParseError, Url};
use watch::Watch;

pub mod fork;
pub mod star;
pub mod watch;

kind!(RepositoryType, Repository);

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum RepositoryAcceptedActivities {
    Watch(Watch),
    Star(Star),
    Fork(Fork),
    OfferIssue(OfferTicket),
    CreateIssueComment(CreateTicketComment),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubRepository {
    #[serde(rename = "type")]
    kind: RepositoryType,
    pub id: ObjectId<Repository>,
    pub name: String,
    pub clone_uri: Url,
    pub attributed_to: Url,
    pub published: chrono::NaiveDateTime,
    pub summary: Option<String>,
    pub forks: Url,
    pub ticket_tracked_by: Url,
    pub send_patches_to: Url,
    pub outbox: Url,
    pub inbox: Url,
    pub followers: Url,
    pub public_key: PublicKey,
}

#[async_trait]
impl GillApubObject for Repository {
    fn view_uri(&self) -> String {
        let attributed_to = self.attributed_to.to_string();
        let username = attributed_to.split('/').last().unwrap();
        format!("/{}/{}", username, self.name)
    }

    async fn followers(&self, instance: &InstanceHandle) -> AppResult<Vec<Url>> {
        let db = instance.database();
        let followers = self.get_watchers(i64::MAX, 0, db).await?;

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

impl Repository {
    pub fn owner_apub_id(&self) -> AppResult<ObjectId<User>> {
        Ok(self.attributed_to.clone().into())
    }

    pub fn activity_pub_id_from_namespace(
        user: &str,
        repository: &str,
    ) -> anyhow::Result<ObjectId<Self>> {
        let domain = &SETTINGS.domain;
        let scheme = if SETTINGS.debug { "http" } else { "https" };
        let url = Url::from_str(&format!(
            "{scheme}://{domain}/apub/users/{user}/repositories/{repository}"
        ))?;
        Ok(ObjectId::new(url))
    }

    pub fn next_item_number(&self) -> i32 {
        self.item_count + 1
    }
}

#[async_trait]
impl ApubObject for Repository {
    type DataType = InstanceHandle;
    type ApubType = ApubRepository;
    type DbType = Repository;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let db = data.database();
        Repository::by_activity_pub_id_optional(object_id.as_str(), db).await
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        let public_key = self.public_key_with_owner()?;
        let forks = Url::parse(&format!("{}/forks", self.activity_pub_id.to_string()))?;

        Ok(ApubRepository {
            kind: Default::default(),
            id: self.activity_pub_id.into(),
            name: self.name,
            clone_uri: self.clone_uri,
            attributed_to: self.attributed_to.clone().into(),
            published: self.published,
            summary: self.summary,
            forks,
            ticket_tracked_by: self.attributed_to.clone().into(),
            outbox: self.outbox_url,
            inbox: self.inbox_url,
            followers: self.followers_url,
            send_patches_to: self.attributed_to.into(),
            public_key,
        })
    }

    async fn from_apub(
        apub: Self::ApubType,
        data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<Self, Self::Error> {
        let db = data.database();
        let repository =
            Repository::by_activity_pub_id_optional(apub.id.inner().as_str(), db).await?;
        if let Some(repository) = repository {
            Ok(repository)
        } else {
            let domain = apub.id.inner().domain().expect("No domain").to_string();
            let repository = CreateRepository {
                activity_pub_id: apub.id.into(),
                name: apub.name,
                summary: apub.summary,
                public_key: apub.public_key.public_key_pem,
                inbox_url: apub.inbox,
                outbox_url: apub.outbox,
                followers_url: apub.followers,
                attributed_to: apub.attributed_to.into(),
                clone_uri: apub.clone_uri,
                send_patches_to: apub.send_patches_to.into(),
                ticket_tracked_by: apub.ticket_tracked_by.into(),
                is_local: false,
                private: false,
                private_key: None,
                domain,
            };

            let repository = repository.save(db).await?;
            Ok(repository)
        }
    }
}

impl Actor for Repository {
    fn public_key(&self) -> &str {
        &self.public_key
    }
    fn inbox(&self) -> Url {
        self.inbox_url.to_owned()
    }
}
