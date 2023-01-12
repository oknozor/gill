use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::error::{AppError};
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::ApubObject;
use activitystreams_kinds::kind;
use async_session::async_trait;
use gill_git::commits::OwnedCommit;
use serde::{Deserialize, Serialize};
use url::Url;

kind!(CommitType, Commit);

#[derive(Debug, Clone)]
pub struct CommitWrapper {
    repository: String,
    owner: String,
    commit: OwnedCommit,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubCommit {
    #[serde(rename = "type")]
    kind: CommitType,
    pub id: ObjectId<CommitWrapper>,
    pub context: ObjectId<Repository>,
    pub attributed_to: ObjectId<User>,
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

#[async_trait]
impl ApubObject for CommitWrapper {
    type DataType = InstanceHandle;
    type ApubType = ApubCommit;
    type DbType = OwnedCommit;
    type Error = AppError;

    async fn read_from_apub_id(
        _object_id: Url,
        _data: &Self::DataType,
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
        _apub: Self::ApubType,
        _data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}
