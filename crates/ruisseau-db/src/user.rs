use crate::repository::Repository;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateSSHKey {
    pub key: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl User {
    pub async fn create(user: CreateUser, pool: &PgPool) -> sqlx::Result<()> {
        let username = user.username;
        let email = user.email;
        let result = sqlx::query!(
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

        println!("{:?}", result);

        Ok(())
    }

    pub async fn by_email(email: &str, pool: &PgPool) -> sqlx::Result<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            select id, username, email from users
            where email = $1
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

    pub async fn get_repository_by_name(
        self,
        repo_name: &str,
        db: &PgPool,
    ) -> sqlx::Result<Repository> {
        let repository = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            select users.id, name, owner_id from users
            join repository r on users.id = r.owner_id
            where r.name = $1
            "#,
            repo_name,
        )
        .fetch_one(db)
        .await?;

        Ok(repository)
    }
}
