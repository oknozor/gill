use crate::domain::pull_request::comment::PullRequestComment;
use crate::error::AppResult;
use std::cmp::Ordering;

use gill_db::repository::pull_request::{
    PullRequest as PullRequestEntity, PullRequestState as PullRequestStateEntity,
};
use gill_git::diffs::Diff;
use gill_git::GitRepository;
use sqlx::PgPool;

pub mod comment;

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

impl PartialOrd<PullRequest> for PullRequest {
    fn partial_cmp(&self, other: &PullRequest) -> Option<Ordering> {
        match (&self.state, &other.state) {
            (PullRequestState::Open, PullRequestState::Closed)
            | (PullRequestState::Open, PullRequestState::Merged) => Some(Ordering::Less),
            (_, _) => Some(self.number.cmp(&other.number)),
        }
    }
}

impl Ord for PullRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PullRequest {
    pub async fn comment(&self, comment: &str, user_id: i32, db: &PgPool) -> AppResult<()> {
        let entity: PullRequestEntity = self.clone().into();
        let comment = comment.escape_default().to_string();
        entity
            .comment(&comment, user_id, db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_comments(&self, db: &PgPool) -> AppResult<Vec<PullRequestComment>> {
        let entity: PullRequestEntity = self.clone().into();
        let comments = entity.get_comments(db).await?;
        Ok(comments.into_iter().map(PullRequestComment::from).collect())
    }

    pub async fn close(&self, db: &PgPool) -> AppResult<()> {
        let entity: PullRequestEntity = self.clone().into();
        entity.close(db).await.map_err(Into::into)
    }

    pub async fn set_merged(&self, db: &PgPool) -> AppResult<()> {
        let entity: PullRequestEntity = self.clone().into();
        entity.set_merged(db).await.map_err(Into::into)
    }

    pub fn get_diff(&self, owner: &str, name: &str) -> AppResult<Vec<Diff>> {
        let repo = GitRepository::open(owner, name)?;
        let diff = repo.diff(&self.base, &self.compare)?;
        Ok(diff)
    }
}
