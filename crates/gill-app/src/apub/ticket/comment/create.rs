use crate::error::AppError;
use crate::instance::InstanceHandle;

use crate::apub::ticket::comment::ApubIssueComment;

use activitypub_federation::deser::helpers::deserialize_one_or_many;

use crate::apub::common::{is_local, GillActivity, GillApubObject};

use crate::domain::issue::Issue;
use crate::domain::user::User;

use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::CreateType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketComment {
    pub(crate) actor: ObjectId<User>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ApubIssueComment,
    #[serde(rename = "type")]
    pub(crate) kind: CreateType,
    pub(crate) id: Url,
}

impl GillActivity for CreateTicketComment {
    fn forward_addresses(&self) -> Vec<&Url> {
        self.to.iter().filter(|url| is_local(url)).collect()
    }
}

#[async_trait]
impl ActivityHandler for CreateTicketComment {
    type DataType = InstanceHandle;
    type Error = AppError;

    fn id(&self) -> &Url {
        self.object.id.inner()
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn receive(
        self,
        instance: &Data<InstanceHandle>,
        request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        if self.object.id.dereference_local(instance).await.is_ok() {
            return Ok(());
        };

        let user = ObjectId::<User>::new(self.actor)
            .dereference_local(instance)
            .await?;

        let comment = self
            .object
            .id
            .dereference(instance, &instance.local_instance, request_counter)
            .await?;

        let comment = comment.save(instance.database()).await?;
        let issue: ObjectId<Issue> = comment.context.into();

        let issue = issue
            .dereference(instance, &instance.local_instance, request_counter)
            .await?;

        issue
            .add_subscriber(user.local_id(), instance.database())
            .await?;

        Ok(())
    }
}
