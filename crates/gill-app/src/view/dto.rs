use gill_db::repository::RepositoryDigest;

pub struct RepositoryDto {
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub star_count: u32,
    pub fork_count: u32,
}

pub struct FederatedRepositoryDto {
    pub owner: String,
    pub name: String,
    pub domain: String,
    pub description: Option<String>,
    pub star_count: u32,
    pub fork_count: u32,
}

impl From<RepositoryDigest> for FederatedRepositoryDto {
    fn from(repo: RepositoryDigest) -> Self {
        FederatedRepositoryDto {
            owner: repo.owner,
            name: repo.name,
            domain: repo.domain,
            description: repo.summary,
            star_count: repo.star_count.map(|c| c as u32).unwrap_or(0),
            fork_count: repo.fork_count.map(|c| c as u32).unwrap_or(0),
        }
    }
}

impl From<RepositoryDigest> for RepositoryDto {
    fn from(repo: RepositoryDigest) -> Self {
        RepositoryDto {
            owner: repo.owner,
            name: repo.name,
            description: repo.summary,
            star_count: repo.star_count.map(|c| c as u32).unwrap_or(0),
            fork_count: repo.fork_count.map(|c| c as u32).unwrap_or(0),
        }
    }
}
