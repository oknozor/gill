use crate::error::AppResult;

use gill_db::repository::digest::RepositoryDigest as RepositoryDigestEntity;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct RepositoryDigest {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub domain: String,
    pub summary: Option<String>,
    pub star_count: Option<i64>,
    pub fork_count: Option<i64>,
    pub watch_count: Option<i64>,
}

impl From<RepositoryDigestEntity> for RepositoryDigest {
    fn from(digest: RepositoryDigestEntity) -> Self {
        Self {
            id: digest.id,
            name: digest.name,
            owner: digest.owner,
            domain: digest.domain,
            summary: digest.summary,
            star_count: digest.star_count,
            fork_count: digest.fork_count,
            watch_count: digest.watch_count,
        }
    }
}

impl RepositoryDigest {
    pub async fn all_local(limit: i64, offset: i64, db: &PgPool) -> AppResult<Vec<Self>> {
        let repositories = RepositoryDigestEntity::all_local(limit, offset, db).await?;
        Ok(repositories
            .into_iter()
            .map(RepositoryDigest::from)
            .collect())
    }

    pub async fn all_federated(limit: i64, offset: i64, db: &PgPool) -> AppResult<Vec<Self>> {
        let repositories = RepositoryDigestEntity::all_federated(limit, offset, db).await?;
        Ok(repositories
            .into_iter()
            .map(RepositoryDigest::from)
            .collect())
    }
}
