use crate::pagination::Pagination;
use crate::repository::Repository;
use sqlx::PgPool;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "pull_request_state")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PullRequest {
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub base: String,
    pub compare: String,
    pub state: PullRequestState,
}

impl Repository {
    pub async fn create_pull_request(
        &self,
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
            INSERT INTO pull_request (number, repository_id, title, description, base, compare)
            VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            self.item_count + 1,
            self.id,
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
                SELECT p.number,
                       p.title,
                       p.description,
                       p.base,
                       p.compare,
                       p.state as "state: PullRequestState"
                FROM pull_request p
                JOIN repository r ON r.id = $1
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
}
