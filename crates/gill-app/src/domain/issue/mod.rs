use crate::domain::id::ActivityPubId;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::error::AppError;

use axum::body::HttpBody;
use chrono::NaiveDateTime;
use gill_db::repository::issue::{Issue as IssueEntity, IssueState as IssueStateEntity};
use gill_db::Insert;

use sqlx::PgPool;

use url::Url;

pub mod comment;
pub mod create;
pub mod digest;

#[derive(Debug, Clone)]
pub struct Issue {
    pub activity_pub_id: ActivityPubId<Issue>,
    pub repository_id: i32,
    pub opened_by: i32,
    pub title: String,
    pub content: String,
    pub state: IssueState,
    pub context: ActivityPubId<Repository>,
    pub attributed_to: ActivityPubId<User>,
    pub media_type: String,
    pub published: NaiveDateTime,
    pub followers_url: Url,
    pub team: Url,
    pub replies: Url,
    pub history: Url,
    pub dependants: Url,
    pub dependencies: Url,
    pub resolved_by: Option<ActivityPubId<User>>,
    pub resolved: Option<NaiveDateTime>,
    pub number: i32,
    pub is_local: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IssueState {
    Open,
    Closed,
}

impl From<IssueStateEntity> for IssueState {
    fn from(state: IssueStateEntity) -> Self {
        match state {
            IssueStateEntity::Open => IssueState::Open,
            IssueStateEntity::Closed => IssueState::Closed,
        }
    }
}

impl From<IssueState> for IssueStateEntity {
    fn from(state: IssueState) -> Self {
        match state {
            IssueState::Open => IssueStateEntity::Open,
            IssueState::Closed => IssueStateEntity::Closed,
        }
    }
}

impl TryFrom<IssueEntity> for Issue {
    type Error = url::ParseError;

    fn try_from(issue: IssueEntity) -> Result<Self, Self::Error> {
        let resolved_by = match issue.resolved_by {
            None => None,
            Some(resolved_by) => {
                let resolved_by = ActivityPubId::try_from(resolved_by)?;
                Some(resolved_by)
            }
        };

        Ok(Self {
            activity_pub_id: ActivityPubId::try_from(issue.activity_pub_id)?,
            repository_id: issue.repository_id,
            opened_by: issue.opened_by,
            title: issue.title,
            content: issue.content,
            state: IssueState::Open,
            context: ActivityPubId::try_from(issue.context)?,
            attributed_to: ActivityPubId::try_from(issue.attributed_to)?,
            media_type: issue.media_type,
            published: Default::default(),
            followers_url: Url::parse(&issue.followers_url)?,
            team: Url::parse(&issue.team)?,
            replies: Url::parse(&issue.replies)?,
            history: Url::parse(&issue.history)?,
            dependants: Url::parse(&issue.dependants)?,
            dependencies: Url::parse(&issue.dependencies)?,
            resolved_by,
            resolved: issue.resolved,
            number: issue.number,
            is_local: issue.is_local,
        })
    }
}

impl From<Issue> for IssueEntity {
    fn from(issue: Issue) -> Self {
        IssueEntity {
            repository_id: issue.repository_id,
            opened_by: issue.opened_by,
            title: issue.title.to_string(),
            content: issue.content.to_string(),
            state: issue.state.into(),
            activity_pub_id: issue.activity_pub_id.to_string(),
            context: issue.context.to_string(),
            attributed_to: issue.attributed_to.to_string(),
            media_type: issue.media_type.clone(),
            published: issue.published,
            followers_url: issue.followers_url.to_string(),
            team: issue.team.to_string(),
            replies: issue.replies.to_string(),
            history: issue.history.to_string(),
            dependants: issue.dependants.to_string(),
            dependencies: issue.dependencies.to_string(),
            resolved_by: issue.resolved_by.map(|resolved_by| resolved_by.to_string()),
            resolved: issue.resolved,
            number: issue.number,
            is_local: issue.is_local,
        }
    }
}

impl From<&Issue> for IssueEntity {
    fn from(val: &Issue) -> Self {
        IssueEntity {
            repository_id: val.repository_id,
            opened_by: val.opened_by,
            title: val.title.to_string(),
            content: val.content.to_string(),
            state: val.state.into(),
            activity_pub_id: val.activity_pub_id.to_string(),
            context: val.context.to_string(),
            attributed_to: val.attributed_to.to_string(),
            media_type: val.media_type.clone(),
            published: val.published,
            followers_url: val.followers_url.to_string(),
            team: val.team.to_string(),
            replies: val.replies.to_string(),
            history: val.history.to_string(),
            dependants: val.dependants.to_string(),
            dependencies: val.dependencies.to_string(),
            resolved_by: val
                .resolved_by
                .as_ref()
                .map(|resolved_by| resolved_by.to_string()),
            resolved: val.resolved,
            number: val.number,
            is_local: val.is_local,
        }
    }
}

impl Issue {
    pub async fn by_activity_pub_id(activity_pub_id: &str, db: &PgPool) -> Result<Self, AppError> {
        let entity = IssueEntity::by_activity_pub_id(activity_pub_id, db).await?;
        Issue::try_from(entity).map_err(Into::into)
    }

    pub async fn by_activity_pub_id_optional(
        activity_pub_id: &str,
        db: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let entity = IssueEntity::by_activity_pub_id(activity_pub_id, db).await;
        match entity {
            Ok(entity) => {
                let issue = Issue::try_from(entity)?;
                Ok(Some(issue))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(AppError::from(err)),
        }
    }

    pub async fn save(self, db: &PgPool) -> Result<Self, AppError> {
        let entity: IssueEntity = self.into();
        let entity = entity.insert(db).await?;
        Issue::try_from(entity).map_err(Into::into)
    }

    pub async fn has_subscriber(&self, subscriber_id: i32, db: &PgPool) -> Result<bool, AppError> {
        let entity = IssueEntity::by_activity_pub_id(&self.activity_pub_id.to_string(), db).await?;

        entity
            .has_subscriber(subscriber_id, db)
            .await
            .map_err(Into::into)
    }

    pub async fn add_subscriber(&self, subscriber_id: i32, db: &PgPool) -> Result<(), AppError> {
        let has_subscriber = self.has_subscriber(subscriber_id, db).await?;
        if !has_subscriber {
            let entity =
                IssueEntity::by_activity_pub_id(&self.activity_pub_id.to_string(), db).await?;

            entity.add_subscriber(subscriber_id, db).await?;
        }

        Ok(())
    }

    async fn followers(&self, db: &PgPool) -> Result<Vec<Url>, AppError> {
        let entity = IssueEntity::by_activity_pub_id(&self.activity_pub_id.to_string(), db).await?;

        let followers = entity
            .get_subscribers_activity_pub_ids(i64::MAX, 0, db)
            .await?;
        let followers = followers
            .into_iter()
            .filter_map(|url| Url::parse(&url).ok())
            .collect();

        Ok(followers)
    }

    pub async fn get_subscribers_inbox(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> Result<Vec<String>, AppError> {
        let entity: IssueEntity = self.into();
        entity
            .get_subscribers_inbox(limit, offset, db)
            .await
            .map_err(Into::into)
    }
}
