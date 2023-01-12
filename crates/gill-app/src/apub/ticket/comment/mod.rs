use crate::apub::common::{GillApubObject, Source};

use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::ApubObject;
use activitystreams_kinds::object::NoteType;
use async_session::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::domain::issue::comment::IssueComment;
use crate::domain::issue::Issue;
use crate::domain::user::User;

use url::Url;
use uuid::Uuid;

pub mod create;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubIssueComment {
    #[serde(rename = "type")]
    pub kind: NoteType,
    pub id: ObjectId<IssueComment>,
    pub attributed_to: ObjectId<User>,
    pub context: ObjectId<Issue>,
    pub in_reply_to: Url,
    pub media_type: String,
    pub content: String,
    pub source: Source,
    pub published: chrono::NaiveDateTime,
}

#[async_trait]
impl ApubObject for IssueComment {
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
        IssueComment::by_activity_pub_id_optional(object_id.as_ref(), db).await
    }

    async fn into_apub(self, _: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(ApubIssueComment {
            kind: NoteType::Note,
            id: self.activity_pub_id.into(),
            attributed_to: self.attributed_to.into(),
            context: self.context.clone().into(),
            in_reply_to: self.context.into(),
            media_type: self.media_type.clone(),
            content: self.content.clone(),
            source: Source {
                content: self.content,
                media_type: self.media_type,
            },
            published: self.published,
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
            activity_pub_id: comment.id.into(),
            number: issue.number,
            repository_id: issue.repository_id,
            created_by: user.local_id(),
            content: comment.content,
            media_type: comment.media_type,
            attributed_to: user.activity_pub_id,
            context: comment.context.into(),
            in_reply_to: comment.attributed_to.into(),
            published: comment.published,
        };

        comment.save(context.database()).await
    }
}
