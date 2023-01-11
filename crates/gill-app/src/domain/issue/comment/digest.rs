use gill_db::repository::issue::comment::IssueCommentDigest as IssueCommentDigestEntity;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct IssueCommentDigest {
    pub id: Uuid,
    pub repository_id: i32,
    pub created_by: String,
    pub content: String,
}

impl From<IssueCommentDigestEntity> for IssueCommentDigest {
    fn from(comment: IssueCommentDigestEntity) -> Self {
        Self {
            id: comment.id,
            repository_id: comment.repository_id,
            created_by: comment.created_by,
            content: comment.content,
        }
    }
}
