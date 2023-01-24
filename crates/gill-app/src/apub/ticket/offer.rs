use crate::error::AppError;
use crate::instance::InstanceHandle;

use crate::apub::ticket::{ApubTicket, TicketType};
use activitypub_federation::deser::helpers::deserialize_one_or_many;

use crate::apub::common::{GillApubObject, Source};
use crate::apub::ticket::accept::AcceptTicket;
use crate::domain::issue::Issue;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use activitypub_federation::traits::{Actor, ApubObject};
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::OfferType;
use axum::async_trait;
use chrono::Utc;
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferTicket {
    /// Activity id
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: OfferType,
    /// The actor sending this activity
    pub actor: ObjectId<User>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    /// The object being offered for publishing
    pub object: ApubTicketOffer,
    /// Indicate under which list/collection/context the sender would like the object to be published
    /// (it may also be the URI of the target actor itself)
    pub target: ObjectId<Repository>,
}

#[async_trait]
impl ActivityHandler for OfferTicket {
    type DataType = InstanceHandle;
    type Error = AppError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    /// Receive an offer ticket activity and automatically send back an accept activity
    /// to the offer sender and the repository followers
    async fn receive(
        self,
        context: &Data<InstanceHandle>,
        request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        let sender = self
            .actor
            .dereference(context, &context.local_instance, request_counter)
            .await?;

        let db = context.database();
        let repository_activity_pub_id = self.target.inner().to_string();
        let repository = Repository::by_activity_pub_id(&repository_activity_pub_id, db).await?;

        let object = self.id.clone();
        let ticket = ApubTicket::from_offer(self, db).await?;
        let issue = Issue::from_apub(ticket, context, request_counter).await?;

        let issue = issue.save(db).await?;
        let hostname = &SETTINGS.domain;
        let id = Url::parse(&format!(
            "https://{hostname}/activity/{uuid}",
            uuid = Uuid::new_v4()
        ))?;
        let repository_owner =
            User::by_activity_pub_id(&repository.attributed_to.clone().to_string(), db).await?;
        let result = issue.activity_pub_id.into();
        let actor = repository_owner.activity_pub_id.clone().into();
        let mut to = repository.followers(context).await?;
        to.push(sender.shared_inbox_or_inbox());
        let recipient = to.clone();

        let accept = AcceptTicket {
            id,
            kind: Default::default(),
            actor,
            to,
            object,
            result,
        };

        repository_owner
            .send(accept, recipient, &context.local_instance)
            .await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubTicketOffer {
    #[serde(rename = "type")]
    pub kind: TicketType,
    pub attributed_to: ObjectId<User>,
    pub summary: String,
    pub media_type: String,
    pub source: Source,
}

impl ApubTicket {
    async fn from_offer(offer: OfferTicket, db: &PgPool) -> Result<Self, AppError> {
        let repository_activity_pub_id = &offer.target.inner().to_string();
        let repository = Repository::by_activity_pub_id(repository_activity_pub_id, db).await?;
        let number = repository.item_count + 1;
        let activity_pub_id = format!("{}/issues/{number}", offer.target.inner());
        let followers = Url::parse(&format!("{activity_pub_id}/followers"))?;
        let team = Url::parse(&format!("{activity_pub_id}/team"))?;
        let replies = Url::parse(&format!("{activity_pub_id}/replies"))?;
        let dependants = Url::parse(&format!("{activity_pub_id}/dependants"))?;
        let dependencies = Url::parse(&format!("{activity_pub_id}/dependencies"))?;
        let history = Url::parse(&format!("{activity_pub_id}/history"))?;
        let id = ObjectId::new(Url::parse(&activity_pub_id)?);

        Ok(ApubTicket {
            kind: Default::default(),
            id,
            context: offer.target,
            attributed_to: offer.object.attributed_to,
            summary: offer.object.summary,
            source: offer.object.source,
            published: Utc::now().naive_utc(),
            followers,
            team,
            replies,
            history,
            dependants,
            dependencies,
            is_resolved: false,
            resolved_by: None,
            resolved: None,
        })
    }
}
