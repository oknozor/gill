use sqlx::PgPool;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Branch {
    pub name: String,
    pub repository_id: i32,
    pub is_default: bool,
}

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
