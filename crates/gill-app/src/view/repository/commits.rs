use crate::error::AppResult;
use crate::oauth::Oauth2User;
use crate::view::repository::{get_repository_branches, BranchDto, Tab};
use crate::view::HtmlTemplate;

use askama::Template;
use axum::extract::Path;
use axum::Extension;

use crate::domain::commit::Author;
use crate::domain::commit::Commit;
use crate::domain::repository::stats::RepositoryStats;
use crate::domain::repository::Repository;
use crate::get_connected_user_username;
use crate::view::filters;

use gill_syntax::diff::diff2html;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/history.html")]
pub struct CommitLogTemplate {
    repository: String,
    owner: String,
    stats: RepositoryStats,
    commits: Vec<Commit>,
    branches: Vec<BranchDto>,
    current_branch: Option<String>,
    user: Option<String>,
    tab: Tab,
}

pub async fn git_log(
    user: Option<Oauth2User>,
    Path((owner, repository, current_branch)): Path<(String, String, String)>,
    Extension(db): Extension<PgPool>,
) -> AppResult<HtmlTemplate<CommitLogTemplate>> {
    let connected_username = get_connected_user_username(&db, user).await;
    let commits = Repository::history(&owner, &repository, &current_branch, &db).await?;
    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;

    Ok(HtmlTemplate(CommitLogTemplate {
        repository,
        owner,
        stats,
        commits,
        branches,
        current_branch: Some(current_branch),
        user: connected_username,
        tab: Tab::History,
    }))
}

#[derive(Template, Debug)]
#[template(path = "repository/commit-diff.html")]
pub struct CommitDiffTemplate {
    // TODO
    _repository: String,
    // TODO
    _owner: String,
    // TODO
    _stats: RepositoryStats,
    commit: Commit,
    diff: String,
    // TODO
    _current_branch: Option<String>,
    user: Option<String>,
    // TODO
    _tab: Tab,
}

pub async fn commit_diff(
    user: Option<Oauth2User>,
    Path((owner, repository, sha)): Path<(String, String, String)>,
    Extension(db): Extension<PgPool>,
) -> AppResult<HtmlTemplate<CommitDiffTemplate>> {
    let connected_username = get_connected_user_username(&db, user).await;
    let (commit, diff) = Repository::commit_with_diff(&owner, &repository, &sha, &db).await?;
    let diff = diff2html(&diff)?;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;

    Ok(HtmlTemplate(CommitDiffTemplate {
        _repository: repository,
        _owner: owner,
        _stats: stats,
        commit,
        diff,
        _current_branch: None,
        user: connected_username,
        _tab: Tab::History,
    }))
}
