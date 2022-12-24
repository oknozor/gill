use crate::repository::Repository;
use crate::user::User;
use sqlx::PgPool;

impl Repository {
    pub async fn add_watcher(&self, watcher_id: i32, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            INSERT INTO repository_watch (repository_id, watched_by)
            VALUES ($1, $2)
            "#,
            self.id,
            watcher_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_watchers(
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
                FROM repository_watch w
                JOIN users u ON w.watched_by = u.id
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
