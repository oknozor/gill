use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

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
    pub is_local: bool,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct RepositoryDigest {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub summary: Option<String>,
    pub star_count: Option<i64>,
    pub fork_count: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateRepository {
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
    pub ticket_tracked_by: String,
    pub send_patches_to: String,
    pub domain: String,
    pub is_local: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Branch {
    pub name: String,
    pub repository_id: i32,
    pub is_default: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Star {
    pub repository_id: i32,
    pub stared_by: i32,
}

impl Repository {
    pub async fn create(repository: &CreateRepository, db: &PgPool) -> sqlx::Result<Repository> {
        let repository = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            insert into repository(
                activity_pub_id,
                name,
                summary,
                private,
                inbox_url,
                outbox_url,
                followers_url,
                attributed_to,
                clone_uri,
                public_key,
                private_key,
                ticket_tracked_by,
                send_patches_to,
                domain,
                is_local
            )
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            returning *;
        "#,
            repository.activity_pub_id,
            repository.name,
            repository.summary,
            repository.private,
            repository.inbox_url,
            repository.outbox_url,
            repository.followers_url,
            repository.attributed_to,
            repository.clone_uri,
            repository.public_key,
            repository.private_key,
            repository.ticket_tracked_by,
            repository.send_patches_to,
            repository.domain,
            repository.is_local,
        )
        .fetch_one(db)
        .await?;

        Ok(repository)
    }

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
                    r.is_local
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

    pub async fn by_activity_pub_id(
        activity_pub_id: &str,
        pool: &PgPool,
    ) -> sqlx::Result<Option<Repository>> {
        let user = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            select * from repository
            where activity_pub_id = $1
            "#,
            activity_pub_id,
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }
}

mod branch {
    use crate::repository::Branch;
    use sqlx::PgPool;

    impl Branch {
        pub(crate) async fn create(
            name: &str,
            repository_id: i32,
            is_default: bool,
            db: &PgPool,
        ) -> sqlx::Result<Branch> {
            let branch = sqlx::query_as!(
                Branch,
                // language=PostgreSQL
                r#"
            insert into "branch"(name, repository_id, is_default)
            values ($1, $2, $3)
            returning name, repository_id, is_default
            "#,
                name,
                repository_id,
                is_default
            )
            .fetch_one(db)
            .await?;

            Ok(branch)
        }

        pub(crate) async fn get(name: &str, repository_id: i32, db: &PgPool) -> Option<Branch> {
            sqlx::query_as!(
                Branch,
                // language=PostgreSQL
                r#"
            SELECT name, repository_id, is_default FROM branch
            WHERE name = $1 AND repository_id = $2
            "#,
                name,
                repository_id
            )
            .fetch_one(db)
            .await
            .ok()
        }

        pub(crate) async fn make_default(self, is_default: bool, db: &PgPool) -> sqlx::Result<()> {
            sqlx::query_as!(
                Branch,
                // language=PostgreSQL
                r#"
            UPDATE branch SET is_default = $1
            WHERE name = $2 AND repository_id = $3
            "#,
                is_default,
                self.name,
                self.repository_id,
            )
            .execute(db)
            .await?;

            Ok(())
        }
    }
}

impl RepositoryDigest {
    pub async fn all_local(
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
                   r.summary,
                   COUNT(rs.repository_id) as star_count,
                   COUNT(rf.repository_id) as fork_count
            FROM repository r
                     RIGHT JOIN users u ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN repository_star rs ON rs.repository_id = r.id
                     LEFT JOIN repository_fork rf ON rf.repository_id = r.id
            WHERE NOT r.private AND r.is_local
            GROUP BY r.id, u.username, r.name, r.id, r.summary
            LIMIT $1 OFFSET $2;"#,
            limit,
            offset,
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
    }

    pub async fn all_federated(
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
                   r.summary,
                   COUNT(rs.repository_id) as star_count,
                   COUNT(rf.repository_id) as fork_count
            FROM repository r
                     RIGHT JOIN users u ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN repository_star rs ON rs.repository_id = r.id
                     LEFT JOIN repository_fork rf ON rf.repository_id = r.id
            WHERE NOT r.private AND NOT r.is_local
            GROUP BY r.id, u.username, r.name, r.id, r.summary
            LIMIT $1 OFFSET $2;"#,
            limit,
            offset,
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
    }
}
