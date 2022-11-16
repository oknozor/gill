use crate::route::repository::{InitRepository, Repository};
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
}
