use crate::domain::id::ActivityPubId;
use crate::domain::issue::Issue;
use crate::domain::user::User;
use crate::error::{AppError, AppResult};
use activitypub_federation::core::object_id::ObjectId;
use gill_db::repository::issue::comment::IssueComment as IssueCommentEntity;
use gill_db::Insert;
use gill_settings::SETTINGS;
use sqlx::PgPool;
use std::str::FromStr;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod create;
pub mod digest;

#[derive(Debug, sqlx::FromRow)]
pub struct IssueComment {
    pub id: Uuid,
    pub activity_pub_id: ActivityPubId<IssueComment>,
    pub number: i32,
    pub repository_id: i32,
    pub created_by: i32,
    pub content: String,
    pub media_type: String,
    pub attributed_to: ActivityPubId<User>,
    pub context: ActivityPubId<Issue>,
    pub in_reply_to: Url,
    pub published: chrono::NaiveDateTime,
}

impl TryFrom<IssueCommentEntity> for IssueComment {
    type Error = ParseError;

    fn try_from(comment: IssueCommentEntity) -> Result<Self, ParseError> {
        Ok(Self {
            id: comment.id,
            activity_pub_id: ActivityPubId::try_from(comment.activity_pub_id)?,
            number: comment.number,
            repository_id: comment.repository_id,
            created_by: comment.created_by,
            content: comment.content.clone(),
            media_type: comment.media_type.clone(),
            attributed_to: ActivityPubId::try_from(comment.attributed_to)?,
            context: ActivityPubId::try_from(comment.context.clone())?,
            in_reply_to: Url::parse(&comment.in_reply_to)?,
            published: comment.published,
        })
    }
}

impl From<&IssueComment> for IssueCommentEntity {
    fn from(val: &IssueComment) -> Self {
        IssueCommentEntity {
            id: val.id,
            activity_pub_id: val.activity_pub_id.to_string(),
            number: val.number,
            repository_id: val.repository_id,
            created_by: val.created_by,
            content: val.content.clone(),
            media_type: val.media_type.clone(),
            attributed_to: val.attributed_to.to_string(),
            context: val.context.to_string(),
            in_reply_to: val.in_reply_to.to_string(),
            published: val.published,
        }
    }
}

impl IssueComment {
    pub async fn by_activity_pub_id_optional(
        activity_pub_id: &str,
        db: &PgPool,
    ) -> AppResult<Option<IssueComment>> {
        let entity = IssueCommentEntity::by_activity_pub_id(activity_pub_id, db).await;
        match entity {
            Ok(entity) => {
                let comment = IssueComment::try_from(entity)?;
                Ok(Some(comment))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(AppError::from(err)),
        }
    }

    pub fn activity_pub_id_from_namespace(
        user: &str,
        repository: &str,
        issue: i32,
        uuid: Uuid,
    ) -> AppResult<ObjectId<Self>> {
        let domain = &SETTINGS.domain;
        let scheme = if SETTINGS.debug { "http" } else { "https" };
        let url = Url::from_str(&format!(
            "{scheme}://{domain}/apub/users/{user}/repositories/{repository}/issues/{issue}/comments/{uuid}"
        ))?;
        Ok(ObjectId::new(url))
    }

    pub async fn save(&self, db: &PgPool) -> AppResult<IssueComment> {
        let entity: IssueCommentEntity = self.into();
        let entity = entity.insert(db).await?;
        IssueComment::try_from(entity).map_err(Into::into)
    }
}
