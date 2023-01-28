use anyhow::Result;
use sqlx::PgPool;

pub async fn inbox_for_url(url: &str, db: &PgPool) -> Result<Vec<String>> {
    gill_db::inbox_for_url(url, db).await.map_err(Into::into)
}
