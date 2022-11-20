use crate::api::repository::{InitRepository, OwnedRepository, Repository};
use sqlx::PgPool;

impl Repository {
    pub async fn create(
        user_id: i32,
        repository: &InitRepository,
        pool: &PgPool,
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
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list(
        limit: i64,
        offset: i64,
        pool: &PgPool,
    ) -> sqlx::Result<Vec<OwnedRepository>> {
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
        .fetch_all(pool)
        .await?;

        Ok(repositories)
    }
}
