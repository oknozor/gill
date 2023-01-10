use crate::error::AppError;
use crate::instance::InstanceHandle;

use crate::apub::ticket::ApubTicket;
use crate::apub::user::UserWrapper;
use activitypub_federation::deser::helpers::deserialize_one_or_many;

use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::CreateType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicket {
    pub(crate) actor: ObjectId<UserWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ApubTicket,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) cc: Vec<Url>,
    #[serde(rename = "type")]
    pub(crate) kind: CreateType,
    pub(crate) id: Url,
}

#[async_trait]
impl ActivityHandler for CreateTicket {
    type DataType = InstanceHandle;
    type Error = AppError;

    fn id(&self) -> &Url {
        self.object.id.inner()
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(
        &self,
        _data: &Data<Self::DataType>,
        _request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(
        self,
        data: &Data<InstanceHandle>,
        request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        ObjectId::<UserWrapper>::new(self.actor)
            .dereference_local(data)
            .await?;

        self.object
            .id
            .dereference(data, &data.local_instance, request_counter)
            .await?;

        Ok(())
    }
}
