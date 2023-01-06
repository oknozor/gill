use crate::apub::repository::RepositoryWrapper;
use crate::apub::user::{ApubUser, UserWrapper};
use crate::apub::GillApubObject;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::core::signatures::PublicKey;
use activitypub_federation::traits::ApubObject;
use activitystreams_kinds::kind;
use async_session::async_trait;
use gill_db::repository::issue::{Issue, IssueDigest, IssueState};
use gill_db::repository::Repository;
use gill_db::user::User;
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};
use serde_json::Value::Object;
use sqlx::PgPool;
use std::fmt::Debug;
use std::str::FromStr;
use url::{ParseError, Url};

pub mod create;

kind!(TicketType, Ticket);

#[derive(Debug)]
pub struct IssueWrapper(Issue);

impl From<Issue> for IssueWrapper {
    fn from(issue: Issue) -> Self {
        Self(issue)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubTicket {
    #[serde(rename = "type")]
    pub kind: TicketType,
    pub id: ObjectId<IssueWrapper>,
    pub context: ObjectId<RepositoryWrapper>,
    pub attributed_to: ObjectId<UserWrapper>,
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
    type DbType = IssueDigest;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let db = data.database();
        let issue = Issue::by_activity_pub_id(&object_id.to_string(), db).await?;
        Ok(issue.map(IssueWrapper))
    }

    async fn into_apub(self, _: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        let context = self.0.context;
        let context = ObjectId::new(Url::parse(&context)?);
        let attributed_to = Url::parse(&self.0.attributed_to)?;
        let attributed_to = ObjectId::new(attributed_to);
        let media_type = self.0.media_type;
        let followers = Url::parse(&self.0.followers_url)?;
        let team = Url::parse(&self.0.team)?;
        let replies = Url::parse(&self.0.replies)?;
        let history = Url::parse(&self.0.replies)?;
        let dependants = Url::parse(&self.0.dependants)?;
        let dependencies = Url::parse(&self.0.dependencies)?;
        let resolved_by = self.0.resolved_by;
        let resolved_by = match resolved_by {
            None => None,
            Some(url) => Some(Url::parse(&url)?),
        };
        let activity_pub_id = Url::parse(&self.0.activity_pub_id)?;

        Ok(ApubTicket {
            kind: Default::default(),
            id: ObjectId::new(activity_pub_id),
            context,
            attributed_to,
            summary: self.0.title,
            source: Source {
                content: self.0.content,
                media_type,
            },
            published: self.0.published,
            followers,
            team,
            replies,
            history,
            dependants,
            dependencies,
            is_resolved: self.0.state == IssueState::Closed,
            resolved_by,
            resolved: self.0.resolved,
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
            .dereference(&context, &context.local_instance, request_counter)
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
            activity_pub_id: ticket.id.inner().to_string(),
            context: ticket.context.inner().to_string(),
            attributed_to: ticket.attributed_to.to_string(),
            media_type: ticket.source.media_type,
            published: ticket.published,
            followers_url: ticket.followers.to_string(),
            team: ticket.team.to_string(),
            replies: ticket.replies.to_string(),
            history: ticket.history.to_string(),
            dependants: ticket.dependants.to_string(),
            dependencies: ticket.dependencies.to_string(),
            resolved_by: ticket.resolved_by.map(|url| url.to_string()),
            resolved: ticket.resolved,
            number: repository.next_item_number(),
            is_local: false,
        };

        let issue = issue.insert(context.database()).await?;
        Ok(IssueWrapper(issue))
    }
}

impl IssueWrapper {
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

    async fn followers(&self, instance: &InstanceHandle) -> Result<Vec<Url>, AppError> {
        let db = instance.database();
        let followers = self.0.get_subscribers(i64::MAX, 0, db).await?;

        let followers = followers
            .into_iter()
            .map(|follower| follower.activity_pub_id)
            .filter_map(|url| Url::parse(&url).ok())
            .collect();

        Ok(followers)
    }
}
