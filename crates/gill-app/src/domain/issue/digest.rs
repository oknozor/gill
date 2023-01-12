use crate::domain::issue::comment::digest::IssueCommentDigest;
use crate::domain::issue::IssueState;
use crate::error::AppResult;
use gill_db::repository::issue::IssueDigest as IssueDigestEntity;
use sqlx::PgPool;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDigest {
    pub repository_id: i32,
    pub number: i32,
    pub opened_by: String,
    pub title: String,
    pub content: String,
    pub state: IssueState,
}

impl From<IssueDigestEntity> for IssueDigest {
    fn from(issue: IssueDigestEntity) -> Self {
        Self {
            repository_id: issue.repository_id,
            number: issue.number,
            opened_by: issue.opened_by,
            title: issue.title,
            content: issue.content,
            state: issue.state.into(),
        }
    }
}

impl From<IssueDigest> for IssueDigestEntity {
    fn from(val: IssueDigest) -> Self {
        IssueDigestEntity {
            repository_id: val.repository_id,
            number: val.number,
            opened_by: val.opened_by,
            title: val.title,
            content: val.content,
            state: val.state.into(),
        }
    }
}

impl PartialOrd<IssueDigest> for IssueDigest {
    fn partial_cmp(&self, other: &IssueDigest) -> Option<Ordering> {
        match (&self.state, &other.state) {
            (IssueState::Open, IssueState::Closed) => Some(Ordering::Less),
            (_, _) => Some(self.number.cmp(&other.number)),
        }
    }
}

impl Ord for IssueDigest {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl IssueDigest {
    pub async fn get_comments(&self, db: &PgPool) -> AppResult<Vec<IssueCommentDigest>> {
        let issue: IssueDigestEntity = self.clone().into();
        let comments = issue.get_comments(db).await?;
        Ok(comments.into_iter().map(IssueCommentDigest::from).collect())
    }
}
