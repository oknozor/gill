use crate::domain::repository::RepositoryStats;
use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::repository::{get_repository_branches, BranchDto};
use crate::view::HtmlTemplate;
use crate::{get_connected_user, get_connected_user_username};
use anyhow::anyhow;
use askama::Template;
use axum::extract::Path;
use axum::response::Redirect;
use axum::{Extension, Form};
use gill_db::repository::Repository;
use serde::Deserialize;

use gill_db::repository::pull_request::PullRequest;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/pulls.html")]
pub struct PullRequestTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_requests: Option<Vec<PullRequest>>,
    stats: RepositoryStats,
    branches: Vec<BranchDto>,
    current_branch: String,
}

pub async fn pulls(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<HtmlTemplate<PullRequestTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let pull_requests = repo.list_pull_requests(&db).await?;
    let pull_requests = (!pull_requests.is_empty()).then_some(pull_requests);
    let current_branch = repo
        .get_default_branch(&db)
        .await
        .ok_or_else(|| anyhow!("No default branch"))?;

    let current_branch = current_branch.name;
    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;

    Ok(HtmlTemplate(PullRequestTemplate {
        user: connected_username,
        owner,
        repository,
        pull_requests,
        stats,
        branches,
        current_branch,
    }))
}

#[derive(Deserialize, Debug)]
pub struct CreatePullRequestForm {
    pub title: String,
    pub description: String,
    pub base: String,
    pub compare: String,
}

pub async fn create(
    connected_user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
    Form(input): Form<CreatePullRequestForm>,
) -> Result<Redirect, AppError> {
    let Some(_user) = get_connected_user(&db, connected_user).await else {
        return Err(AppError::Unauthorized);
    };

    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    repo.create_pull_request(
        &input.title,
        Some(&input.description),
        &input.base,
        &input.compare,
        &db,
    )
    .await?;
    Ok(Redirect::to(&format!("/{owner}/{repository}/pulls")))
}
