use crate::apub::common::{GillApubObject, Source};

use crate::apub::ticket::IssueWrapper;
use crate::apub::user::UserWrapper;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;

use activitypub_federation::traits::ApubObject;

use activitystreams_kinds::object::NoteType;
use async_session::async_trait;
use gill_db::repository::issue::comment::IssueComment;

use gill_db::Insert;

use serde::{Deserialize, Serialize};

use std::fmt::Debug;
use std::str::FromStr;

use gill_settings::SETTINGS;
use url::Url;
use uuid::Uuid;

pub mod create;

#[derive(Debug)]
pub struct IssueCommentWrapper(IssueComment);

impl From<IssueComment> for IssueCommentWrapper {
    fn from(comment: IssueComment) -> Self {
        Self(comment)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubIssueComment {
    #[serde(rename = "type")]
    pub kind: NoteType,
    pub id: ObjectId<IssueCommentWrapper>,
    pub attributed_to: ObjectId<UserWrapper>,
    pub context: ObjectId<IssueWrapper>,
    pub in_reply_to: Url,
    pub media_type: String,
    pub content: String,
    pub source: Source,
    pub published: chrono::NaiveDateTime,
}

#[async_trait]
impl ApubObject for IssueCommentWrapper {
    type DataType = InstanceHandle;
    type ApubType = ApubIssueComment;
    type DbType = IssueComment;
    type Error = AppError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let db = data.database();
        let comment = IssueComment::by_activity_pub_id(object_id.as_ref(), db).await?;
        Ok(comment.map(IssueCommentWrapper))
    }

    async fn into_apub(self, _: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(ApubIssueComment {
            kind: NoteType::Note,
            id: ObjectId::new(Url::parse(&self.0.activity_pub_id)?),
            attributed_to: ObjectId::new(Url::parse(&self.0.attributed_to)?),
            context: ObjectId::new(Url::parse(&self.0.context)?),
            in_reply_to: Url::parse(&self.0.context)?,
            media_type: self.0.media_type.clone(),
            content: self.0.content.clone(),
            source: Source {
                content: self.0.content,
                media_type: self.0.media_type,
            },
            published: self.0.published,
        })
    }

    async fn verify(
        _: &Self::ApubType,
        _: &Url,
        _: &Self::DataType,
        _: &mut i32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn from_apub(
        comment: Self::ApubType,
        context: &Self::DataType,
        request_counter: &mut i32,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let user = comment
            .attributed_to
            .dereference(context, &context.local_instance, request_counter)
            .await?;

        let issue = comment
            .context
            .dereference(context, &context.local_instance, request_counter)
            .await?;

        let comment = IssueComment {
            id: Uuid::new_v4(),
            activity_pub_id: comment.id.inner().to_string(),
            number: issue.0.number,
            repository_id: issue.0.repository_id,
            created_by: user.local_id(),
            content: comment.content,
            media_type: comment.media_type,
            attributed_to: user.activity_pub_id_as_url()?.to_string(),
            context: comment.context.inner().to_string(),
            in_reply_to: comment.attributed_to.inner().to_string(),
            published: comment.published,
        };

        let comment = comment.insert(context.database()).await?;
        Ok(IssueCommentWrapper::from(comment))
    }
}

impl IssueCommentWrapper {
    pub fn activity_pub_id_from_namespace(
        user: &str,
        repository: &str,
        issue: i32,
        uuid: Uuid,
    ) -> anyhow::Result<ObjectId<Self>> {
        let domain = &SETTINGS.domain;
        let scheme = if SETTINGS.debug { "http" } else { "https" };
        let url = Url::from_str(&format!(
            "{scheme}://{domain}/apub/users/{user}/repositories/{repository}/issues/{issue}/comments/{uuid}"
        ))?;
        Ok(ObjectId::new(url))
    }
}
