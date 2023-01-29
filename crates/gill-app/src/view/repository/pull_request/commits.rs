use crate::domain::commit::Author;
use crate::domain::commit::Commit;
use crate::domain::pull_request::PullRequest;
use crate::domain::pull_request::PullRequestState;
use crate::domain::repository::stats::RepositoryStats;
use crate::domain::repository::Repository;
use crate::error::AppError;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;
use crate::view::filters;
use crate::view::repository::Tab;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::Path;
use axum::Extension;

use gill_syntax::diff::diff2html;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/pulls/commits.html")]
pub struct PullRequestCommitsTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_request: PullRequest,
    stats: RepositoryStats,
    current_branch: Option<String>,
    commits: Vec<Commit>,
    tab: Tab,
}

pub async fn commits(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<HtmlTemplate<PullRequestCommitsTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let pull_request = repo.get_pull_request(pull_request_number, &db).await?;
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);
    let commits =
        Repository::get_commits_for_pull_request(&owner, &repository, &pull_request, &db).await?;

    Ok(HtmlTemplate(PullRequestCommitsTemplate {
        user: connected_username,
        owner: owner.clone(),
        repository: repository.clone(),
        pull_request,
        stats,
        current_branch,
        commits,
        tab: Tab::PullRequests,
    }))
}

#[derive(Template, Debug)]
#[template(path = "repository/pulls/commit-diff.html")]
pub struct PullRequestCommitDiffTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_request: PullRequest,
    stats: RepositoryStats,
    current_branch: Option<String>,
    commit: Commit,
    diff: String,
    tab: Tab,
}

pub async fn commit_diff(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number, sha)): Path<(String, String, i32, String)>,
) -> Result<HtmlTemplate<PullRequestCommitDiffTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let pull_request = repo.get_pull_request(pull_request_number, &db).await?;
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);
    let (commit, diff) = Repository::commit_with_diff(&owner, &repository, &sha, &db).await?;
    let diffs = diff2html(&diff)?;

    Ok(HtmlTemplate(PullRequestCommitDiffTemplate {
        user: connected_username,
        owner,
        repository,
        pull_request,
        stats,
        current_branch,
        commit,
        diff: diffs,
        tab: Tab::PullRequests,
    }))
}
