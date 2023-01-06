use crate::repository::issue::Issue;
use crate::user::User;
use sqlx::PgPool;

impl Issue {
    pub async fn add_subscriber(&self, subscriber_id: i32, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            insert into "issue_subscriber"(repository_id, number, subscriber)
            values ($1, $2, $3)
            "#,
            self.repository_id,
            self.number,
            subscriber_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_subscribers(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<User>> {
        let followers = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
                SELECT u.id, username, domain, email, public_key, private_key, inbox_url, outbox_url,
                followers_url, is_local, activity_pub_id
                FROM issue_subscriber s
                JOIN users u ON s.subscriber = u.id
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
            .fetch_all(db)
            .await?;

        Ok(followers)
    }
}
