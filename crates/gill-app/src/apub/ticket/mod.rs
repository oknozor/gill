use crate::apub::common::{GillApubObject, Source};
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;

use activitypub_federation::traits::ApubObject;
use activitystreams_kinds::kind;
use async_session::async_trait;
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};

use crate::domain::id::ActivityPubId;
use crate::domain::issue::{Issue, IssueState};
use crate::domain::repository::Repository;
use crate::domain::user::User;
use std::fmt::Debug;
use std::str::FromStr;
use url::Url;

pub mod accept;
pub mod comment;
pub mod offer;

kind!(TicketType, Ticket);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubTicket {
    #[serde(rename = "type")]
    pub kind: TicketType,
    pub id: ObjectId<Issue>,
    pub context: ObjectId<Repository>,
    pub attributed_to: ObjectId<User>,
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
    pub resolved_by: Option<Url>,
    pub resolved: Option<chrono::NaiveDateTime>,
}

#[async_trait]
impl ApubObject for Issue {
    type DataType = InstanceHandle;
    type ApubType = ApubTicket;
    type DbType = Issue;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let db = data.database();
        Issue::by_activity_pub_id_optional(object_id.as_ref(), db).await
    }

    async fn into_apub(self, _: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(ApubTicket {
            kind: Default::default(),
            id: self.activity_pub_id.into(),
            context: self.context.into(),
            attributed_to: self.attributed_to.into(),
            summary: self.title,
            source: Source {
                content: self.content,
                media_type: self.media_type,
            },
            published: self.published,
            followers: self.followers_url,
            team: self.team,
            replies: self.replies,
            history: self.history,
            dependants: self.dependants,
            dependencies: self.dependencies,
            is_resolved: self.state == IssueState::Closed,
            resolved_by: self.resolved_by.map(Into::into),
            resolved: self.resolved,
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
        ticket: ApubTicket,
        context: &InstanceHandle,
        request_counter: &mut i32,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let user = ticket
            .attributed_to
            .dereference(context, &context.local_instance, request_counter)
            .await?;

        let repository = ticket
            .context
            .dereference(context, &context.local_instance, request_counter)
            .await?;

        let issue = Issue {
            repository_id: repository.local_id(),
            opened_by: user.local_id(),
            title: ticket.summary,
            content: ticket.source.content,
            state: if ticket.is_resolved {
                IssueState::Closed
            } else {
                IssueState::Closed
            },
            activity_pub_id: ActivityPubId::from(ticket.id),
            context: ActivityPubId::from(ticket.context),
            attributed_to: ActivityPubId::from(ticket.attributed_to),
            media_type: ticket.source.media_type,
            published: ticket.published,
            followers_url: ticket.followers,
            team: ticket.team,
            replies: ticket.replies,
            history: ticket.history,
            dependants: ticket.dependants,
            dependencies: ticket.dependencies,
            resolved_by: ticket.resolved_by.map(ActivityPubId::from),
            resolved: ticket.resolved,
            number: repository.next_item_number(),
            is_local: false,
        };

        let issue = issue.save(context.database()).await?;
        Ok(issue)
    }
}

impl Issue {
    pub fn activity_pub_id_from_namespace(
        user: &str,
        repository: &str,
        number: i32,
    ) -> anyhow::Result<ObjectId<Self>> {
        let domain = &SETTINGS.domain;
        let scheme = if SETTINGS.debug { "http" } else { "https" };
        let url = Url::from_str(&format!(
            "{scheme}://{domain}/apub/users/{user}/repositories/{repository}/issues/{number}"
        ))?;

        Ok(ObjectId::new(url))
    }
}
