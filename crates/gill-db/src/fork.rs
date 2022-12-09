use crate::repository::Repository;
use crate::user::User;
use sqlx::PgPool;

impl Repository {
    pub async fn add_fork(&self, fork_id: i32, forked_by: i32, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            INSERT INTO gill.public.repository_fork (repository_id, fork_id, forked_by)
            VALUES ($1, $2, $3)
            "#,
            self.id,
            fork_id,
            forked_by
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_forked_by(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<User>> {
        let watchers = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
                SELECT u.id, username, domain, email, public_key, private_key, inbox_url, outbox_url,
                followers_url, is_local, activity_pub_id
                FROM repository_fork f
                JOIN users u ON f.forked_by = u.id
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
            .fetch_all(db)
            .await?;

        Ok(watchers)
    }
}
