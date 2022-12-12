use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::repository::{get_repository_branches, BranchDto};
use crate::view::HtmlTemplate;
use anyhow::Result;
use askama::Template;
use axum::extract::Path;
use axum::Extension;

use crate::get_connected_user_username;
use gill_db::repository::RepositoryLight;
use gill_git::repository::commits::OwnedCommit;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/history.html")]
pub struct CommitHistoryTemplate {
    repository: String,
    owner: String,
    watch_count: u32,
    fork_count: u32,
    star_count: u32,
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
    let commits = gill_git::repository::commits::history(&owner, &repository)?;
    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;
    let stats = RepositoryLight::stats_by_namespace(&owner, &repository, &db).await?;

    Ok(HtmlTemplate(CommitHistoryTemplate {
        repository,
        owner,
        watch_count: stats.watch_count.unwrap_or(0) as u32,
        fork_count: stats.fork_count.unwrap_or(0) as u32,
        star_count: stats.watch_count.unwrap_or(0) as u32,
        commits,
        branches,
        current_branch,
        user: connected_username,
    }))
}
