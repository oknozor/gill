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

use crate::apub::object::repository::RepositoryWrapper;
use crate::apub::object::user::UserWrapper;
use axum::async_trait;
use gill_db::repository::{CreateRepository, Repository};
use gill_git::repository::commits::OwnedCommit;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CommitWrapper {
    repository: String,
    owner: String,
    commit: OwnedCommit,
}

impl From<OwnedCommit> for CommitWrapper {
    fn from(commit: OwnedCommit) -> Self {
        CommitWrapper(commit)
    }
}

kind!(CommitType, Commit);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubCommit {
    #[serde(rename = "type")]
    kind: CommitType,
    pub id: ObjectId<CommitWrapper>,
    pub context: ObjectId<RepositoryWrapper>,
    pub attributed_to: ObjectId<UserWrapper>,
    pub created: chrono::NaiveDateTime,
    pub committed: chrono::NaiveDateTime,
    pub hash: String,
    pub summary: String,
    pub description: CommitDescription,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDescription {
    media_type: String,
    content: String,
}

impl CommitDescription {
    fn new(content: String) -> Self {
        Self {
            media_type: "text/plain".to_string(),
            content,
        }
    }
}

impl CommitWrapper {
    fn activity_pub_id(&self) -> &str {
        &self.0.activity_pub_id
    }

    pub fn local_id(&self) -> i32 {
        self.0.id
    }

    fn activity_pub_id_as_url(&self) -> Result<Url, AppError> {
        Ok(Url::parse(self.activity_pub_id())?)
    }
}

#[async_trait]
impl ApubObject for CommitWrapper {
    type DataType = InstanceHandle;
    type ApubType = ApubCommit;
    type DbType = OwnedCommit;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        todo!()
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        todo!()
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
        todo!()
    }
}
