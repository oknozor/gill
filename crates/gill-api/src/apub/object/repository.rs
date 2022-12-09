use crate::apub::activities::follow::Follow;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::{
    core::{activity_queue::send_activity, object_id::ObjectId, signatures::PublicKey},
    deser::context::WithContext,
    traits::{ActivityHandler, Actor, ApubObject},
    LocalInstance,
};
use activitystreams_kinds::kind;
use std::str::FromStr;

use axum::async_trait;
use gill_db::repository::{CreateRepository, Repository};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RepositoryWrapper(Repository);

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

impl RepositoryWrapper {
    pub async fn followers(&self, instance: &InstanceHandle) -> Result<Vec<Url>, AppError> {
        let db = instance.database();
        let followers = self.0.get_watchers(i64::MAX, 0, db).await?;

        let followers = followers
            .into_iter()
            .map(|follower| follower.activity_pub_id)
            .filter_map(|url| Url::parse(&url).ok())
            .collect();

        Ok(followers)
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
        other: &RepositoryWrapper,
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
        let user = Repository::by_activity_pub_id(object_id.as_str(), db).await?;
        Ok(user.map(RepositoryWrapper))
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        let id = self.activity_pub_id_as_url()?;
        let owner_url = Url::from_str(&self.0.attributed_to)?;
        let public_key = self.public_key()?;

        Ok(ApubRepository {
            kind: Default::default(),
            id: ObjectId::new(id.clone()),
            name: self.0.name,
            clone_uri: Url::from_str(&self.0.clone_uri)?,
            attributed_to: owner_url.clone(),
            published: self.0.published,
            summary: self.0.summary,
            forks: id.join("fork")?,
            ticket_tracked_by: owner_url.clone(),
            outbox: id.join("outbox")?,
            inbox: id.join("inbox")?,
            followers: id.join("followers")?,
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
