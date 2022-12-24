use crate::repository::Repository;
use crate::user::User;
use sqlx::PgPool;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Star {
    pub repository_id: i32,
    pub stared_by: i32,
}

impl Repository {
    pub async fn add_star(&self, starred_by: i32, db: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            INSERT INTO repository_star (repository_id, starred_by)
            VALUES ($1, $2)
            "#,
            self.id,
            starred_by
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_starred_by(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<User>> {
        let stars = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
                SELECT u.id, username, domain, email, public_key, private_key, inbox_url, outbox_url,
                followers_url, is_local, activity_pub_id
                FROM repository_star s
                JOIN users u ON s.starred_by = u.id
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
            .fetch_all(db)
            .await?;

        Ok(stars)
    }
}
