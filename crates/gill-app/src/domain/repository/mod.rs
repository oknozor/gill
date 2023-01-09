use gill_db::repository::digest::{RepositoryDigest, RepositoryLight};
use sqlx::PgPool;
use serde::{Serialize, Deserialize};
use crate::apub::repository::RepositoryWrapper;
use crate::apub::user::UserWrapper;
use crate::domain::ActivityPubId;

#[derive(Deserialize, Serialize, Debug)]
pub struct Repository {
    pub id: i32,
    pub activity_pub_id: ActivityPubId<RepositoryWrapper>,
    pub name: String,
    pub summary: Option<String>,
    pub private: bool,
    pub inbox_url: String,
    pub outbox_url: String,
    pub followers_url: String,
    pub attributed_to: ActivityPubId<UserWrapper>,
    pub clone_uri: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub published: chrono::NaiveDateTime,
    pub ticket_tracked_by: ActivityPubId<RepositoryWrapper>,
    pub send_patches_to: String,
    pub domain: String,
    pub item_count: i32,
    pub is_local: bool,
}


#[derive(Debug)]
pub struct RepositoryStats {
    pub fork_count: u32,
    pub star_count: u32,
    pub watch_count: u32,
}

impl From<RepositoryLight> for RepositoryStats {
    fn from(stats: RepositoryLight) -> Self {
        Self {
            watch_count: stats.watch_count.unwrap_or(0) as u32,
            fork_count: stats.fork_count.unwrap_or(0) as u32,
            star_count: stats.star_count.unwrap_or(0) as u32,
        }
    }
}

impl From<&RepositoryDigest> for RepositoryStats {
    fn from(repo: &RepositoryDigest) -> Self {
        Self {
            watch_count: repo.watch_count.unwrap_or(0) as u32,
            fork_count: repo.fork_count.unwrap_or(0) as u32,
            star_count: repo.star_count.unwrap_or(0) as u32,
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
