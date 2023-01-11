use crate::domain::pull_request::comment::PullRequestComment;
use crate::error::AppError;

use gill_db::repository::pull_request::{
    PullRequest as PullRequestEntity, PullRequestState as PullRequestStateEntity,
};
use sqlx::PgPool;

pub mod comment;

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub repository_id: i32,
    pub number: i32,
    pub opened_by: String,
    pub title: String,
    pub description: Option<String>,
    pub base: String,
    pub compare: String,
    pub state: PullRequestState,
}

#[derive(Debug, Copy, Clone)]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

impl From<PullRequestState> for PullRequestStateEntity {
    fn from(state: PullRequestState) -> Self {
        match state {
            PullRequestState::Open => PullRequestStateEntity::Open,
            PullRequestState::Closed => PullRequestStateEntity::Closed,
            PullRequestState::Merged => PullRequestStateEntity::Merged,
        }
    }
}

impl From<PullRequestStateEntity> for PullRequestState {
    fn from(state: PullRequestStateEntity) -> Self {
        match state {
            PullRequestStateEntity::Open => PullRequestState::Open,
            PullRequestStateEntity::Closed => PullRequestState::Closed,
            PullRequestStateEntity::Merged => PullRequestState::Merged,
        }
    }
}

impl From<PullRequest> for PullRequestEntity {
    fn from(val: PullRequest) -> Self {
        PullRequestEntity {
            repository_id: val.repository_id,
            number: val.number,
            opened_by: val.opened_by,
            title: val.title,
            description: val.description,
            base: val.base,
            compare: val.compare,
            state: val.state.into(),
        }
    }
}

impl From<PullRequestEntity> for PullRequest {
    fn from(pull_request: PullRequestEntity) -> Self {
        Self {
            repository_id: pull_request.repository_id,
            number: pull_request.number,
            opened_by: pull_request.opened_by,
            title: pull_request.title,
            description: pull_request.description,
            base: pull_request.base,
            compare: pull_request.compare,
            state: pull_request.state.into(),
        }
    }
}

impl PullRequest {
    pub async fn comment(&self, comment: &str, user_id: i32, db: &PgPool) -> Result<(), AppError> {
        let entity: PullRequestEntity = self.clone().into();
        entity
            .comment(comment, user_id, db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_comments(&self, db: &PgPool) -> Result<Vec<PullRequestComment>, AppError> {
        let entity: PullRequestEntity = self.clone().into();
        let comments = entity.get_comments(db).await?;
        Ok(comments.into_iter().map(PullRequestComment::from).collect())
    }

    pub async fn close(&self, db: &PgPool) -> Result<(), AppError> {
        let entity: PullRequestEntity = self.clone().into();
        entity.close(db).await.map_err(Into::into)
    }

    pub async fn merged(&self, db: &PgPool) -> Result<(), AppError> {
        let entity: PullRequestEntity = self.clone().into();
        entity.merged(db).await.map_err(Into::into)
    }
}
