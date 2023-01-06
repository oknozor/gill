use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::ApubObject;
use activitystreams_kinds::kind;
use async_session::async_trait;
use gill_db::repository::issue::Issue;
use serde::{Deserialize, Serialize};
use url::Url;

kind!(TicketType, Ticket);

#[derive(Debug)]
pub struct IssueWrapper(Issue);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubTicket {
    #[serde(rename = "type")]
    kind: TicketType,
    pub id: ObjectId<IssueWrapper>,
    pub context: Url,
    pub attributed_to: Url,
    pub summary: String,
    pub source: Source,
    pub published: chrono::NaiveDateTime,
    pub followers: Url,
    pub team: Url,
    pub replies: Url,
    pub history: Url,
    pub dependants: Url,
    pub dependencies: Url,
    pub is_resolved: bool,
    pub resolved_by: Url,
    pub resolved: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    content: String,
    media_type: String,
}

#[async_trait]
impl ApubObject for IssueWrapper {
    type DataType = InstanceHandle;
    type ApubType = ApubTicket;
    type DbType = Issue;
    type Error = anyhow::Error;

    async fn read_from_apub_id(
        _object_id: Url,
        _data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
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
        todo!()
    }

    async fn from_apub(
        _apub: Self::ApubType,
        _data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        todo!()
    }
}
