use crate::repository::issue::IssueDigest;
use crate::Insert;
use async_trait::async_trait;
use sqlx::types::Uuid;
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub struct IssueCommentDigest {
    pub id: Uuid,
    pub repository_id: i32,
    pub created_by: String,
    pub content: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct IssueComment {
    pub id: Uuid,
    pub activity_pub_id: String,
    pub number: i32,
    pub repository_id: i32,
    pub created_by: i32,
    pub content: String,
    pub media_type: String,
    pub attributed_to: String,
    pub context: String,
    pub in_reply_to: String,
    pub published: chrono::NaiveDateTime,
}

#[async_trait]
impl Insert for IssueComment {
    type Output = Self;

    async fn insert(self, db: &PgPool) -> sqlx::Result<Self> {
        let comment = sqlx::query_as!(
            IssueComment,
            // language=PostgreSQL
            r#"
           INSERT INTO issue_comment (id,
                                      activity_pub_id,
                                      number,
                                      repository_id,
                                      created_by,
                                      content,
                                      media_type,
                                      attributed_to,
                                      context,
                                      in_reply_to,
                                      published)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING id, activity_pub_id, number, repository_id, created_by, content, media_type, attributed_to, context, in_reply_to, published;"#,
            self.id,
            self.activity_pub_id,
            self.number,
            self.repository_id,
            self.created_by,
            self.content,
            self.media_type,
            self.attributed_to,
            self.context,
            self.in_reply_to,
            self.published,
        )
            .fetch_one(db)
            .await?;

        Ok(comment)
    }
}
impl IssueComment {
    pub async fn by_activity_pub_id(
        activity_pub_id: &str,
        db: &PgPool,
    ) -> sqlx::Result<Option<Self>> {
        let user = sqlx::query_as!(
            IssueComment,
            // language=PostgreSQL
            r#"
            select * from issue_comment
            where activity_pub_id = $1
            "#,
            activity_pub_id,
        )
        .fetch_optional(db)
        .await?;

        Ok(user)
    }
}

impl IssueDigest {
    pub async fn get_comments(&self, db: &PgPool) -> sqlx::Result<Vec<IssueCommentDigest>> {
        let comments = sqlx::query_as!(
            IssueCommentDigest,
            // language=PostgreSQL
            r#"
           SELECT c.id, c.repository_id, u.username as created_by, c.content FROM issue_comment c
                JOIN users u on u.id = c.created_by
                WHERE c.repository_id = $1
                AND c.number = $2;
           "#,
            self.repository_id,
            self.number,
        )
        .fetch_all(db)
        .await?;

        Ok(comments)
    }
}
