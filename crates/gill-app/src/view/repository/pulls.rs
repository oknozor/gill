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

use gill_db::repository::pull_request::{PullRequest, PullRequestComment};
use gill_git::GitRepository;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/pulls.html")]
pub struct PullRequestsTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_requests: Option<Vec<PullRequest>>,
    stats: RepositoryStats,
    branches: Vec<BranchDto>,
    current_branch: String,
}

pub async fn list_view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<HtmlTemplate<PullRequestsTemplate>, AppError> {
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

    Ok(HtmlTemplate(PullRequestsTemplate {
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
    let Some(user) = get_connected_user(&db, connected_user).await else {
        return Err(AppError::Unauthorized);
    };

    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    repo.create_pull_request(
        user.id,
        &input.title,
        Some(&input.description),
        &input.base,
        &input.compare,
        &db,
    )
    .await?;
    Ok(Redirect::to(&format!("/{owner}/{repository}/pulls")))
}

#[derive(Template, Debug)]
#[template(path = "repository/pulls/pull.html")]
pub struct PullRequestTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_request: PullRequest,
    stats: RepositoryStats,
    branches: Vec<BranchDto>,
    current_branch: String,
    comments: Vec<PullRequestComment>,
}

pub async fn view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<HtmlTemplate<PullRequestTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let pull_request = repo.get_pull_request(pull_request_number, &db).await?;
    let comments = pull_request.get_comments(&db).await?;
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
        pull_request,
        stats,
        branches,
        current_branch,
        comments,
    }))
}

#[derive(Deserialize, Debug)]
pub struct CommentPullRequestForm {
    pub comment: String,
}

pub async fn comment(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
    Form(input): Form<CommentPullRequestForm>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    let comment = input.comment.escape_default().to_string();
    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .get_pull_request(pull_request_number, &db)
        .await?
        .comment(&comment, user.id, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}

pub async fn rebase(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    let repository_entity = Repository::by_namespace(&owner, &repository, &db).await?;

    if repository_entity.attributed_to != user.activity_pub_id {
        return Err(AppError::Unauthorized);
    };

    let pull_request = repository_entity
        .get_pull_request(pull_request_number, &db)
        .await?;

    let git_repository = GitRepository::open(&owner, &repository)?;

    git_repository.rebase(
        &pull_request.base,
        &pull_request.compare,
        &user.username,
        user.email.as_ref().unwrap(),
    )?;

    pull_request.close(&db).await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}

pub async fn merge(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    let repository_entity = Repository::by_namespace(&owner, &repository, &db).await?;

    if repository_entity.attributed_to != user.activity_pub_id {
        return Err(AppError::Unauthorized);
    };

    let pull_request = repository_entity
        .get_pull_request(pull_request_number, &db)
        .await?;

    let git_repository = GitRepository::open(&owner, &repository)?;

    // TODO: make email mandatory
    git_repository.merge(
        &pull_request.base,
        &pull_request.compare,
        &user.username,
        user.email.as_ref().unwrap(),
    )?;

    pull_request.close(&db).await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}

pub async fn close(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    let repository_entity = Repository::by_namespace(&owner, &repository, &db).await?;

    if repository_entity.attributed_to != user.activity_pub_id {
        return Err(AppError::Unauthorized);
    };

    repository_entity
        .get_pull_request(pull_request_number, &db)
        .await?
        .close(&db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}
