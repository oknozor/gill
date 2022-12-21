use crate::domain::repository::RepositoryStats;
use gill_db::repository::RepositoryDigest;

pub struct RepositoryDto {
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub stats: RepositoryStats,
}

pub struct FederatedRepositoryDto {
    pub owner: String,
    pub name: String,
    pub domain: String,
    pub description: Option<String>,
    pub stats: RepositoryStats,
}

impl From<RepositoryDigest> for FederatedRepositoryDto {
    fn from(repo: RepositoryDigest) -> Self {
        let stats = RepositoryStats::from(&repo);
        FederatedRepositoryDto {
            owner: repo.owner,
            name: repo.name,
            domain: repo.domain,
            description: repo.summary,
            stats,
        }
    }
}

impl From<RepositoryDigest> for RepositoryDto {
    fn from(repo: RepositoryDigest) -> Self {
        let stats = RepositoryStats::from(&repo);
        RepositoryDto {
            owner: repo.owner,
            name: repo.name,
            description: repo.summary,
            stats,
        }
    }
}
