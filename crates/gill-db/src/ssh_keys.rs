use crate::user::User;
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, FromRow)]
pub struct SshKey {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub key_type: String,
    pub owner_id: i32,
}

impl SshKey {
    pub async fn get(ssh_key: &str, key_type: &str, pool: &PgPool) -> sqlx::Result<Option<SshKey>> {
        let key = sqlx::query_as!(
            SshKey,
            // language=PostgreSQL
            r#"
            select * from ssh_key
            where key_type = $1 AND key = $2;
            "#,
            key_type,
            ssh_key,
        )
        .fetch_optional(pool)
        .await?;

        Ok(key)
    }
}

impl User {
    pub async fn add_ssh_key(
        user_id: i32,
        key_name: &str,
        ssh_key: &str,
        key_type: &str,
        pool: &PgPool,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            // language=PostgreSQL
            r#"
            insert into "ssh_key"(owner_id, key, name, key_type)
            values ($1, $2, $3, $4)
        "#,
            user_id,
            ssh_key,
            key_name,
            key_type,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
