use crate::pagination::Pagination;
use crate::repository::Repository;
use comment::PullRequestComment;
use sqlx::PgPool;

pub mod comment;

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "pull_request_state")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

#[derive(sqlx::FromRow, Debug)]
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

impl PullRequest {
    pub async fn comment(&self, comment: &str, user_id: i32, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
           INSERT INTO pull_request_comment (number, repository_id, created_by, content)
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

    pub async fn get_comments(&self, db: &PgPool) -> sqlx::Result<Vec<PullRequestComment>> {
        let comments = sqlx::query_as!(
            PullRequestComment,
           // language=PostgreSQL
           r#"
           SELECT c.id, c.repository_id, u.username as created_by, c.content FROM pull_request_comment c
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
           UPDATE pull_request SET state = 'Closed'
            WHERE pull_request.number = $1 AND repository_id = $2;
           "#,
            self.number,
            self.repository_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn set_merged(&self, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
           UPDATE pull_request SET state = 'Merged'
            WHERE pull_request.number = $1 AND repository_id = $2;
           "#,
            self.number,
            self.repository_id
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

impl Repository {
    pub async fn create_pull_request(
        &self,
        user_id: i32,
        title: &str,
        description: Option<&str>,
        base: &str,
        compare: &str,
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
            INSERT INTO pull_request (number, repository_id, opened_by, title, description, base, compare)
            VALUES ($1, $2, $3, $4, $5, $6, $7);
            "#,
            self.item_count + 1,
            self.id,
            user_id,
            title,
            description,
            base,
            compare
        )
            .execute(&mut transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn list_pull_requests(&self, db: &PgPool) -> sqlx::Result<Vec<PullRequest>> {
        let pagination = Pagination::default();
        let pull_requests = sqlx::query_as!(
            PullRequest,
            // language=PostgreSQL
            r#"
                SELECT
                        P.repository_id,
                        p.number,
                        u.username as opened_by,
                        p.title,
                        p.description,
                        p.base,
                        p.compare,
                        p.state as "state: PullRequestState"
                FROM pull_request p
                JOIN repository r ON r.id = $1
                JOIN users u on u.id = p.opened_by
                WHERE r.id = p.repository_id
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

    pub async fn get_pull_request(&self, number: i32, db: &PgPool) -> sqlx::Result<PullRequest> {
        let pull_request = sqlx::query_as!(
            PullRequest,
            // language=PostgreSQL
            r#"
                SELECT
                    p.repository_id,
                    p.number,
                    u.username as opened_by,
                    p.title,
                    p.description,
                    p.base,
                    p.compare,
                    p.state as "state: PullRequestState"
                FROM pull_request p
                JOIN users u on u.id = p.opened_by
                WHERE number = $1 AND repository_id = $2
            "#,
            number,
            self.id,
        )
        .fetch_one(db)
        .await?;

        Ok(pull_request)
    }
}
