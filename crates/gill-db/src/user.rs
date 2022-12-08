use crate::repository::Repository;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateSSHKey {
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateUser {
    pub username: String,
    pub email: Option<String>,
    pub private_key: Option<String>,
    pub public_key: String,
    pub activity_pub_id: String,
    pub outbox_url: String,
    pub inbox_url: String,
    pub domain: String,
    pub followers_url: String,
    pub is_local: bool,
}

/// A user living in gill database
#[derive(Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Debug, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub domain: String,
    pub email: Option<String>,
    pub public_key: String,
    pub private_key: Option<String>,
    pub activity_pub_id: String,
    pub inbox_url: String,
    pub outbox_url: String,
    pub followers_url: String,
    pub is_local: bool,
}

impl User {
    pub async fn create(user: CreateUser, pool: &PgPool) -> sqlx::Result<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            insert into "users"(
                                username,
                                email,
                                domain,
                                inbox_url,
                                outbox_url,
                                followers_url,
                                private_key,
                                public_key,
                                is_local,
                                activity_pub_id)
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            returning *;
        "#,
            user.username,
            user.email,
            user.domain,
            user.inbox_url,
            user.outbox_url,
            user.followers_url,
            user.private_key,
            user.public_key,
            user.is_local,
            user.activity_pub_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn by_email(email: &str, pool: &PgPool) -> sqlx::Result<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            select * from users
            where email = $1
            "#,
            Some(email),
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn by_activity_pub_id(
        activity_pub_id: &str,
        pool: &PgPool,
    ) -> sqlx::Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            select * from users
            where activity_pub_id = $1
            "#,
            activity_pub_id,
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn by_user_name(username: &str, pool: &PgPool) -> sqlx::Result<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            select * from users
            where username = $1
            "#,
            username,
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
            SELECT
                users.id,
                name,
                r.description,
                r.private,
                owner_id
            FROM users
            JOIN repository r ON users.id = r.owner_id
            WHERE r.name = $1
            "#,
            repo_name,
        )
        .fetch_one(db)
        .await?;

        Ok(repository)
    }

    pub async fn list_repositories(self, db: &PgPool) -> sqlx::Result<Vec<Repository>> {
        let repository = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            SELECT
                users.id,
                name,
                r.description,
                r.private,
                owner_id
            FROM users
            JOIN repository r ON users.id = r.owner_id
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(repository)
    }
}
