use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::repository::{get_repository_branches, BranchDto};
use crate::view::HtmlTemplate;
use anyhow::Result;
use askama::Template;
use axum::extract::Path;
use axum::Extension;

use crate::domain::repository::RepositoryStats;
use crate::get_connected_user_username;

use gill_git::commits::OwnedCommit;
use gill_git::GitRepository;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/history.html")]
pub struct CommitHistoryTemplate {
    repository: String,
    owner: String,
    stats: RepositoryStats,
    commits: Vec<OwnedCommit>,
    branches: Vec<BranchDto>,
    current_branch: String,
    user: Option<String>,
}

pub async fn history(
    user: Option<Oauth2User>,
    Path((owner, repository, current_branch)): Path<(String, String, String)>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<CommitHistoryTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let repo = GitRepository::open(&owner, &repository)?;
    let commits = repo.history()?;
    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;

    Ok(HtmlTemplate(CommitHistoryTemplate {
        repository,
        owner,
        stats,
        commits,
        branches,
        current_branch,
        user: connected_username,
    }))
}
