use crate::domain::issue::comment::digest::IssueCommentDigest;
use crate::domain::issue::IssueState;
use crate::error::AppError;
use gill_db::repository::issue::IssueDigest as IssueDigestEntity;
use sqlx::PgPool;

#[derive(Debug, Clone)]
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

impl IssueDigest {
    pub async fn get_comments(&self, db: &PgPool) -> Result<Vec<IssueCommentDigest>, AppError> {
        let issue: IssueDigestEntity = self.clone().into();
        let comments = issue.get_comments(db).await?;
        Ok(comments.into_iter().map(IssueCommentDigest::from).collect())
    }
}
