use branch::Branch;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

pub mod branch;
pub mod create;
pub mod digest;
pub mod fork;
pub mod issue;
pub mod pull_request;
pub mod star;
pub mod watch;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, FromRow)]
pub struct Repository {
    pub id: i32,
    pub activity_pub_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub private: bool,
    pub inbox_url: String,
    pub outbox_url: String,
    pub followers_url: String,
    pub attributed_to: String,
    pub clone_uri: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub published: chrono::NaiveDateTime,
    pub ticket_tracked_by: String,
    pub send_patches_to: String,
    pub domain: String,
    pub item_count: i32,
    pub is_local: bool,
}

impl Repository {
    pub async fn by_namespace(owner: &str, name: &str, db: &PgPool) -> sqlx::Result<Repository> {
        let repository = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            SELECT  r.id,
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
            FROM repository r
            JOIN users u on r.attributed_to = u.activity_pub_id
            WHERE u.username = $1 AND r.name = $2
            "#,
            owner,
            name
        )
        .fetch_one(db)
        .await?;

        Ok(repository)
    }

    pub async fn list_branches(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<Branch>> {
        let repositories = sqlx::query_as!(
            Branch,
            // language=PostgreSQL
            r#"
                SELECT b.name, b.repository_id, b.is_default FROM repository r
                JOIN branch b ON b.repository_id = r.id
                WHERE r.id = $3
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset,
            self.id
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
    }

    pub async fn set_default_branch(&self, branch_name: &str, db: &PgPool) -> sqlx::Result<()> {
        // If we have a current default branch, make it no default
        let current_default = self.get_default_branch(db).await;
        if let Some(current_default) = current_default {
            current_default.make_default(false, db).await?;
        }

        // Create or update the default branch
        let branch_exists = Branch::get(branch_name, self.id, db).await;
        match branch_exists {
            None => {
                Branch::create(branch_name, self.id, true, db).await?;
            }
            Some(branch) => branch.make_default(true, db).await?,
        }

        Ok(())
    }

    pub async fn get_default_branch(&self, db: &PgPool) -> Option<Branch> {
        sqlx::query_as!(
            Branch,
            // language=PostgreSQL
            r#"
            SELECT b.name, repository_id, is_default FROM branch b
            JOIN repository r on r.id = b.repository_id
            WHERE repository_id = $1 AND b.is_default
            "#,
            self.id
        )
        .fetch_one(db)
        .await
        .ok()
    }

    pub async fn create_branch(&self, branch_name: &str, db: &PgPool) -> sqlx::Result<()> {
        Branch::create(branch_name, self.id, false, db).await?;
        Ok(())
    }

    pub async fn by_activity_pub_id(
        activity_pub_id: &str,
        pool: &PgPool,
    ) -> sqlx::Result<Repository> {
        let user = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            select * from repository
            where activity_pub_id = $1
            "#,
            activity_pub_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn owner(&self, db: &PgPool) -> sqlx::Result<String> {
        let user = sqlx::query!(
            // language=PostgreSQL
            r#"
            select username from repository r
            JOIN users u ON r.attributed_to = r.activity_pub_id
            where r.id = $1

            "#,
            self.id,
        )
        .fetch_one(db)
        .await?;

        Ok(user.username)
    }

    pub async fn by_id(id: i32, pool: &PgPool) -> sqlx::Result<Repository> {
        let user = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            select * from repository
            where id = $1
            "#,
            id,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}
