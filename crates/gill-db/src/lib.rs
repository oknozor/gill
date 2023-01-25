use async_trait::async_trait;

use sqlx::PgPool;

pub mod pagination;
pub mod repository;
pub mod subscribe;
pub mod user;

pub use sqlx::postgres::PgPoolOptions;

#[async_trait]
pub trait Insert {
    type Output;
    async fn insert(self, db: &PgPool) -> sqlx::Result<Self::Output>;
}

pub async fn inbox_for_url(url: &str, db: &PgPool) -> sqlx::Result<Vec<String>> {
    let inboxes = sqlx::query_scalar!(
        // language=PostgreSQL
        r#"
            SELECT DISTINCT member.inbox_url
            FROM users u
                     LEFT JOIN repository r ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN user_follow uf on u.id = uf.user_id AND r.id IS NULL
                     LEFT JOIN repository_watch rw on r.id = rw.repository_id AND r.id IS NOT NULL
                     JOIN users as member on member.id = uf.follower_id OR member.id = rw.watched_by
                WHERE r.followers_url = $1
                   OR u.followers_url = $1
            "#,
        url,
    )
    .fetch_all(db)
    .await?;

    Ok(inboxes)
}
