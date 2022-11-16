use crate::route::user::{CreateUser, User};
use sqlx::PgPool;

impl User {
    pub async fn create(user: CreateUser, pool: &PgPool) -> sqlx::Result<()> {
        let username = user.username;
        let email = user.email;
        sqlx::query!(
            // language=PostgreSQL
            r#"
            insert into "users"(username, email)
            values ($1, $2)
        "#,
            username,
            email
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn by_email(email: &str, pool: &PgPool) -> sqlx::Result<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            select * from users
            where email =  ($1)
        "#,
            email,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn add_ssh_key(user_id: i32, ssh_key: &str, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            insert into "public_key"(owner_id, key)
            values ($1, $2)
        "#,
            user_id,
            ssh_key
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
