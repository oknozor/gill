#[derive(Debug, sqlx::FromRow)]
pub struct PullRequestComment {
    pub id: i32,
    pub repository_id: i32,
    pub created_by: String,
    pub content: String,
}
