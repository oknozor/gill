use crate::domain::repository::digest::RepositoryDigest;
use gill_db::repository::digest::RepositoryLight;
use sqlx::PgPool;

#[derive(Debug)]
pub struct RepositoryStats {
    pub fork_count: u32,
    pub star_count: u32,
    pub watch_count: u32,
    pub clone_url: String,
}

impl From<RepositoryLight> for RepositoryStats {
    fn from(stats: RepositoryLight) -> Self {
        Self {
            watch_count: stats.watch_count.unwrap_or(0) as u32,
            fork_count: stats.fork_count.unwrap_or(0) as u32,
            star_count: stats.star_count.unwrap_or(0) as u32,
            clone_url: stats.clone_url,
        }
    }
}

impl From<&RepositoryDigest> for RepositoryStats {
    fn from(repo: &RepositoryDigest) -> Self {
        Self {
            watch_count: repo.watch_count.unwrap_or(0) as u32,
            fork_count: repo.fork_count.unwrap_or(0) as u32,
            star_count: repo.star_count.unwrap_or(0) as u32,
            clone_url: repo.clone_url.clone(),
        }
    }
}

impl RepositoryStats {
    pub async fn get(
        owner: &str,
        repository: &str,
        db: &PgPool,
    ) -> anyhow::Result<RepositoryStats> {
        let repo = RepositoryLight::stats_by_namespace(owner, repository, db).await?;
        Ok(RepositoryStats::from(repo))
    }
}
