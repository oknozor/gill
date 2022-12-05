use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Debug, FromRow)]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub owner_id: i32,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq, Debug, FromRow)]
pub struct OwnedRepository {
    pub id: i32,
    pub owner_id: i32,
    pub name: String,
    pub owner_name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct InitRepository {
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Branch {
    pub name: String,
    pub repository_id: i32,
    pub is_default: bool,
}

impl Repository {
    pub async fn create(
        user_id: i32,
        repository: &InitRepository,
        db: &PgPool,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            insert into "repository"(name, owner_id)
            values ($1, $2)
        "#,
            repository.name,
            user_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn by_namespace(owner: &str, name: &str, db: &PgPool) -> sqlx::Result<Repository> {
        let repository = sqlx::query_as!(
            Repository,
            // language=PostgreSQL
            r#"
            SELECT r.id, r.name, owner_id FROM repository r
            JOIN users u on r.owner_id = u.id
            WHERE u.username = $1 AND r.name = $2

            "#,
            owner,
            name
        )
        .fetch_one(db)
        .await?;

        Ok(repository)
    }

    pub async fn list(limit: i64, offset: i64, db: &PgPool) -> sqlx::Result<Vec<OwnedRepository>> {
        let repositories = sqlx::query_as!(
            OwnedRepository,
            // language=PostgreSQL
            r#"
                SELECT r.id, r.owner_id, r.name, u.username as owner_name FROM repository r
                JOIN users u ON u.id = r.owner_id
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
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
