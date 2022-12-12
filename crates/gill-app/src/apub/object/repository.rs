use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::{
    core::{object_id::ObjectId, signatures::PublicKey},
    traits::{ActivityHandler, Actor, ApubObject},
};
use activitystreams_kinds::kind;
use std::str::FromStr;

use crate::apub::activities::fork::Fork;
use crate::apub::activities::star::Star;
use crate::apub::activities::watch::Watch;

use crate::apub::object::user::UserWrapper;
use crate::apub::object::GillApubObject;
use activitypub_federation::data::Data;
use axum::async_trait;
use gill_db::repository::{CreateRepository, Repository};
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

#[derive(Debug, Clone)]
pub struct RepositoryWrapper(Repository);

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum RepositoryAcceptedActivities {
    Watch(Watch),
    Star(Star),
    Fork(Fork),
}

impl From<Repository> for RepositoryWrapper {
    fn from(repository: Repository) -> Self {
        RepositoryWrapper(repository)
    }
}

kind!(RepositoryType, Repository);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubRepository {
    #[serde(rename = "type")]
    kind: RepositoryType,
    pub id: ObjectId<RepositoryWrapper>,
    pub name: String,
    pub clone_uri: String,
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
impl GillApubObject for RepositoryWrapper {
    fn view_uri(&self) -> String {
        let username = self.0.attributed_to.split('/').last().unwrap();
        format!("/{}/{}", username, self.0.name)
    }

    fn followers_url(&self) -> Result<Url, AppError> {
        Url::parse(&self.0.followers_url).map_err(Into::into)
    }

    async fn followers(&self, instance: &InstanceHandle) -> Result<Vec<Url>, AppError> {
        let db = instance.database();
        let followers = self.0.get_watchers(i64::MAX, 0, db).await?;

        let followers = followers
            .into_iter()
            .map(|follower| follower.activity_pub_id)
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

impl RepositoryWrapper {
    pub fn owner_apub_id(&self) -> Result<ObjectId<UserWrapper>, AppError> {
        Ok(ObjectId::new(Url::parse(&self.0.attributed_to)?))
    }

    pub async fn add_watcher(
        &self,
        watcher_id: i32,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let db = instance.database();
        self.0.add_watcher(watcher_id, db).await?;
        Ok(())
    }

    pub async fn add_fork(
        &self,
        forked_by: i32,
        fork: i32,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let db = instance.database();
        self.0.add_fork(fork, forked_by, db).await?;
        Ok(())
    }

    pub async fn add_star(
        &self,
        starred_by: i32,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let db = instance.database();
        self.0.add_star(starred_by, db).await?;
        Ok(())
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
}

#[async_trait]
impl ApubObject for RepositoryWrapper {
    type DataType = InstanceHandle;
    type ApubType = ApubRepository;
    type DbType = RepositoryWrapper;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let db = data.database();
        let repository = Repository::by_activity_pub_id(object_id.as_str(), db).await?;
        Ok(repository.map(RepositoryWrapper))
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        let id = self.activity_pub_id_as_url()?;
        let owner_url = Url::from_str(&self.0.attributed_to)?;
        let public_key = self.public_key_with_owner()?;
        let forks = Url::parse(&format!("{id}/forks"))?;

        Ok(ApubRepository {
            kind: Default::default(),
            id: ObjectId::new(id),
            name: self.0.name,
            clone_uri: self.0.clone_uri,
            attributed_to: owner_url.clone(),
            published: self.0.published,
            summary: self.0.summary,
            forks,
            ticket_tracked_by: owner_url.clone(),
            outbox: Url::parse(&self.0.outbox_url)?,
            inbox: Url::parse(&self.0.inbox_url)?,
            followers: Url::parse(&self.0.followers_url)?,
            send_patches_to: owner_url,
            public_key,
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
        println!("{:?}", apub);
        let db = data.database();
        let id = Url::from(apub.id);
        let repository = Repository::by_activity_pub_id(id.as_str(), db).await?;
        if let Some(repository) = repository {
            Ok(RepositoryWrapper(repository))
        } else {
            let repository = CreateRepository {
                activity_pub_id: id.to_string(),
                name: apub.name,
                summary: apub.summary,
                public_key: apub.public_key.public_key_pem,
                inbox_url: apub.inbox.to_string(),
                outbox_url: apub.outbox.to_string(),
                followers_url: apub.followers.to_string(),
                attributed_to: apub.attributed_to.to_string(),
                clone_uri: apub.clone_uri.to_string(),
                send_patches_to: apub.send_patches_to.to_string(),
                ticket_tracked_by: apub.ticket_tracked_by.to_string(),
                is_local: false,
                private: false,
                private_key: None,
                domain: id.domain().expect("No domain").to_string(),
            };

            let repository = Repository::create(&repository, db).await?;
            Ok(RepositoryWrapper(repository))
        }
    }
}

impl Actor for RepositoryWrapper {
    fn public_key(&self) -> &str {
        &self.0.public_key
    }
    fn inbox(&self) -> Url {
        Url::from_str(&self.0.inbox_url).expect("Invalid inbox url")
    }
}
