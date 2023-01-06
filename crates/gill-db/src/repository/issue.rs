use crate::pagination::Pagination;
use crate::repository::Repository;
use sqlx::PgPool;
use std::fmt::Debug;

#[derive(Debug, sqlx::Type, Eq, PartialEq)]
#[sqlx(type_name = "pull_request_state")]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(Debug)]
pub struct IssueDigest {
    pub repository_id: i32,
    pub number: i32,
    pub opened_by: String,
    pub title: String,
    pub content: String,
    pub state: IssueState,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Issue {
    pub repository_id: i32,
    pub opened_by: i32,
    pub title: String,
    pub content: String,
    pub state: IssueState,
    pub activity_pub_id: String,
    pub context: String,
    pub attributed_to: String,
    pub media_type: String,
    pub published: chrono::NaiveDateTime,
    pub followers_url: String,
    pub team: String,
    pub replies: String,
    pub history: String,
    pub dependants: String,
    pub dependencies: String,
    pub resolved_by: Option<String>,
    pub resolved: Option<chrono::NaiveDateTime>,
    pub number: i32,
    pub is_local: bool,
}

impl Issue {
    pub async fn insert(self, db: &PgPool) -> Result<Issue, sqlx::Error> {
        let mut transaction = db.begin().await?;
        sqlx::query!(
            // language=PostgreSQL
            r#"
                UPDATE repository
                SET item_count = item_count + 1
                WHERE id = $1
                "#,
            self.repository_id,
        )
        .execute(&mut transaction)
        .await?;

        let issue = sqlx::query_as!(
            Issue,
            // language=PostgreSQL
            r#"
            INSERT INTO issue (repository_id,
                                opened_by,
                                title,
                                content,
                                activity_pub_id,
                                context,
                                attributed_to,
                                media_type,
                                followers_url,
                                team,
                                replies,
                                history,
                                dependants,
                                dependencies,
                                resolved_by,
                                number,
                                is_local)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING repository_id,
                opened_by,
                title,
                content,
                state as "state: IssueState",
                activity_pub_id,
                context,
                attributed_to,
                media_type,
                published,
                followers_url,
                team,
                replies,
                history,
                dependants,
                dependencies,
                resolved_by,
                resolved,
                number,
                is_local;"#,
            self.repository_id,
            self.opened_by,
            self.title,
            self.content,
            self.activity_pub_id,
            self.context,
            self.attributed_to,
            self.media_type,
            self.followers_url,
            self.team,
            self.replies,
            self.history,
            self.dependants,
            self.dependencies,
            self.resolved_by,
            self.number,
            self.is_local,
        )
        .fetch_one(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(issue)
    }

    pub async fn by_activity_pub_id(
        activity_pub_id: &str,
        db: &PgPool,
    ) -> Result<Option<Issue>, sqlx::Error> {
        let issue = sqlx::query_as!(
            Issue,
            // language=PostgreSQL
            r#"
                SELECT
                    repository_id,
                    opened_by,
                    title,
                    content,
                    state as "state: IssueState",
                    activity_pub_id,
                    context,
                    attributed_to,
                    media_type,
                    published,
                    followers_url,
                    team,
                    replies,
                    history,
                    dependants,
                    dependencies,
                    resolved_by,
                    resolved,
                    number,
                    is_local
                FROM issue
                WHERE activity_pub_id = $1
            "#,
            activity_pub_id,
        )
        .fetch_optional(db)
        .await?;

        Ok(issue)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct IssueComment {
    pub id: i32,
    pub repository_id: i32,
    pub created_by: String,
    pub content: String,
}

impl Repository {
    pub async fn list_issues(&self, db: &PgPool) -> sqlx::Result<Vec<IssueDigest>> {
        let pagination = Pagination::default();
        let pull_requests = sqlx::query_as!(
            IssueDigest,
            // language=PostgreSQL
            r#"
                SELECT
                        i.repository_id,
                        i.number,
                        u.username as opened_by,
                        i.title,
                        i.content,
                        i.state as "state: IssueState"
                FROM issue i
                JOIN repository r ON r.id = $1
                JOIN users u on u.id = i.opened_by
                LIMIT $2
                OFFSET $3
            "#,
            self.id,
            pagination.limit,
            pagination.offset,
        )
        .fetch_all(db)
        .await?;

        Ok(pull_requests)
    }

    pub async fn create_issue(
        &self,
        user_id: i32,
        title: &str,
        content: &str,
        db: &PgPool,
    ) -> sqlx::Result<()> {
        let mut transaction = db.begin().await?;
        sqlx::query!(
            // language=PostgreSQL
            r#"
                UPDATE repository
                SET item_count = $1
                WHERE id = $2
                "#,
            self.item_count + 1,
            self.id,
        )
        .execute(&mut transaction)
        .await?;

        sqlx::query!(
            // language=PostgreSQL
            r#"
            INSERT INTO issue (number, repository_id, opened_by, title, content)
            VALUES ($1, $2, $3, $4, $5);
            "#,
            self.item_count + 1,
            self.id,
            user_id,
            title,
            content,
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_issue(&self, number: i32, db: &PgPool) -> sqlx::Result<IssueDigest> {
        let issue = sqlx::query_as!(
            IssueDigest,
            // language=PostgreSQL
            r#"
                SELECT
                    i.repository_id,
                    i.number,
                    u.username as opened_by,
                    i.title,
                    i.content,
                    i.state as "state: IssueState"
                FROM issue i
                JOIN users u on u.id = i.opened_by
                WHERE number = $1 AND repository_id = $2
            "#,
            number,
            self.id,
        )
        .fetch_one(db)
        .await?;

        Ok(issue)
    }
}

impl IssueDigest {
    pub async fn comment(&self, comment: &str, user_id: i32, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
           INSERT INTO issue_comment (number, repository_id, created_by, content)
           VALUES ($1, $2, $3, $4);
           "#,
            self.number,
            self.repository_id,
            user_id,
            comment,
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_comments(&self, db: &PgPool) -> sqlx::Result<Vec<IssueComment>> {
        let comments = sqlx::query_as!(
            IssueComment,
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

    pub async fn close(&self, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
           UPDATE issue SET state = 'Closed'
            WHERE issue.number = $1 AND repository_id = $2;
           "#,
            self.number,
            self.repository_id
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
