use crate::repository::digest::RepositoryDigest;
use crate::repository::Repository;
use crate::Insert;
use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

pub mod follow;
pub mod ssh_keys;

pub struct CreateSSHKey {
    pub name: String,
    pub key: String,
}

#[derive(Debug, Clone)]
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
#[derive(Clone, PartialEq, Eq, Debug, FromRow)]
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

#[async_trait]
impl Insert for CreateUser {
    type Output = User;

    async fn insert(self, db: &PgPool) -> sqlx::Result<Self::Output> {
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
            self.username,
            self.email,
            self.domain,
            self.inbox_url,
            self.outbox_url,
            self.followers_url,
            self.private_key,
            self.public_key,
            self.is_local,
            self.activity_pub_id,
        )
        .fetch_one(db)
        .await?;

        Ok(user)
    }
}

impl User {
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

    pub async fn by_id(id: i32, pool: &PgPool) -> sqlx::Result<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            select * from users
            where id = $1
            "#,
            id,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn by_activity_pub_id(activity_pub_id: &str, pool: &PgPool) -> sqlx::Result<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            r#"
            select * from users
            where activity_pub_id = $1
            "#,
            activity_pub_id,
        )
        .fetch_one(pool)
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

    pub async fn get_local_repository_by_name(
        &self,
        repo_name: &str,
        db: &PgPool,
    ) -> sqlx::Result<Repository> {
        let repository = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            SELECT
                r.id,
                r.activity_pub_id,
                r.name,
                r.summary,
                r.private,
                r.inbox_url,
                r.outbox_url,
                r.followers_url,
                r.attributed_to,
                r.clone_uri,
                r.public_key,
                r.private_key,
                r.published,
                r.ticket_tracked_by,
                r.send_patches_to,
                r.domain,
                r.is_local,
                r.item_count
            FROM users u
            JOIN repository r ON u.activity_pub_id = r.attributed_to
            WHERE r.name = $1 AND r.is_local
            "#,
            repo_name,
        )
        .fetch_one(db)
        .await?;

        Ok(repository)
    }

    pub async fn list_repositories(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<RepositoryDigest>> {
        let repositories = sqlx::query_as!(
            RepositoryDigest,
            // language=PostgreSQL
            r#"
            SELECT r.id,
                   r.name,
                   u.username as owner,
                   r.domain,
                   r.summary,
                   COUNT(rs.repository_id) as star_count,
                   COUNT(rf.repository_id) as fork_count,
                   COUNT(rw.repository_id) as watch_count,
                   r.clone_uri as clone_url
            FROM repository r
                     RIGHT JOIN users u ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN repository_star rs ON rs.repository_id = r.id
                     LEFT JOIN repository_fork rf ON rf.repository_id = r.id
                     LEFT JOIN repository_watch rw ON rw.repository_id = r.id
            WHERE NOT r.private AND r.is_local AND r.attributed_to = $1
            GROUP BY r.id, u.username, r.name, r.id, r.summary
            LIMIT $2 OFFSET $3;"#,
            self.activity_pub_id,
            limit,
            offset,
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
    }

    pub async fn list_starred_repositories(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<RepositoryDigest>> {
        let repositories = sqlx::query_as!(
            RepositoryDigest,
            // language=PostgreSQL
            r#"
            SELECT r.id,
                   r.name,
                   u.username as owner,
                   r.domain,
                   r.summary,
                   COUNT(rs.repository_id) as star_count,
                   COUNT(rf.repository_id) as fork_count,
                   COUNT(rw.repository_id) as watch_count,
                   r.clone_uri as clone_url
            FROM repository r
                     RIGHT JOIN users u ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN repository_star rs ON rs.repository_id = r.id
                     LEFT JOIN repository_fork rf ON rf.repository_id = r.id
                     LEFT JOIN repository_watch rw ON rw.repository_id = r.id
            WHERE NOT r.private AND rs.starred_by = $1
            GROUP BY r.id, u.username, r.name, r.id, r.summary
            LIMIT $2 OFFSET $3;"#,
            self.id,
            limit,
            offset,
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
    }
}
