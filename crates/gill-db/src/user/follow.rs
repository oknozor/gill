use crate::user::User;
use sqlx::PgPool;

impl User {
    pub async fn add_follower(&self, follower_id: i32, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            insert into "user_follow"(
                                user_id,
                                follower_id)
            values ($1, $2)
            "#,
            self.id,
            follower_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_followers(
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
                FROM user_follow f
                JOIN users u ON f.user_id = u.id
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
