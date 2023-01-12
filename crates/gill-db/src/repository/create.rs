use crate::repository::Repository;
use crate::Insert;
use async_trait::async_trait;
use sqlx::PgPool;

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

#[async_trait]
impl Insert for CreateRepository {
    type Output = Repository;

    async fn insert(self, db: &PgPool) -> sqlx::Result<Repository> {
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
            self.activity_pub_id,
            self.name,
            self.summary,
            self.private,
            self.inbox_url,
            self.outbox_url,
            self.followers_url,
            self.attributed_to,
            self.clone_uri,
            self.public_key,
            self.private_key,
            self.ticket_tracked_by,
            self.send_patches_to,
            self.domain,
            self.is_local,
        )
        .fetch_one(db)
        .await?;

        Ok(repository)
    }
}
