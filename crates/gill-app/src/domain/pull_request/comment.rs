use gill_db::repository::pull_request::comment::PullRequestComment as CommentEntity;

#[derive(Debug, Clone)]
pub struct PullRequestComment {
    pub id: i32,
    pub repository_id: i32,
    pub created_by: String,
    pub content: String,
}

impl From<CommentEntity> for PullRequestComment {
    fn from(comment: CommentEntity) -> Self {
        Self {
            id: comment.id,
            repository_id: comment.repository_id,
            created_by: comment.created_by,
            content: comment.content,
        }
    }
}
