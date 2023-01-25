use crate::error::AppError;
use crate::instance::InstanceHandle;

use activitypub_federation::deser::helpers::deserialize_one_or_many;

use crate::domain::issue::Issue;

use crate::apub::common::{is_local, GillActivity};
use crate::domain::repository::Repository;
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::AcceptType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptTicket {
    /// Activity id
    pub(crate) id: Url,
    #[serde(rename = "type")]
    pub(crate) kind: AcceptType,
    /// The repository managing this ticket
    pub(crate) actor: ObjectId<Repository>,
    /// Collection of this repository follower's inboxes and the
    /// offer author inbox
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    // Todo: make this accept the whole offer object as well
    /// the offer activity or its id
    pub(crate) object: Url,
    /// The accepted ticket
    pub(crate) result: ObjectId<Issue>,
}

impl GillActivity for AcceptTicket {
    fn forward_addresses(&self) -> Vec<&Url> {
        self.to.iter().filter(|url| is_local(url)).collect()
    }
}

#[async_trait]
impl ActivityHandler for AcceptTicket {
    type DataType = InstanceHandle;
    type Error = AppError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn receive(
        self,
        data: &Data<InstanceHandle>,
        request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        ObjectId::<Repository>::new(self.actor)
            .dereference(data, &data.local_instance, request_counter)
            .await?;

        let issue = self
            .result
            .dereference(data, &data.local_instance, request_counter)
            .await?;

        issue.save(data.database()).await?;

        Ok(())
    }
}
