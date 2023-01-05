use crate::domain::repository::RepositoryStats;
use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::component::MarkdownPreviewForm;
use crate::view::repository::{get_repository_branches, BranchDto};
use crate::view::HtmlTemplate;
use crate::{get_connected_user, get_connected_user_username};
use anyhow::anyhow;
use askama::Template;
use axum::extract::Path;
use axum::response::Redirect;
use axum::Extension;
use gill_db::repository::pull_request::PullRequestState;
use gill_db::repository::pull_request::{PullRequest, PullRequestComment};
use gill_db::repository::Repository;
use gill_git::GitRepository;
use sqlx::PgPool;

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
    markdown_preview_form: MarkdownPreviewForm,
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
    let action_href = format!(
        "/{owner}/{repository}/pulls/{}/comment",
        pull_request.number
    );

    Ok(HtmlTemplate(PullRequestTemplate {
        user: connected_username,
        owner: owner.clone(),
        repository: repository.clone(),
        pull_request,
        stats,
        branches,
        current_branch,
        comments,
        markdown_preview_form: MarkdownPreviewForm {
            action_href,
            submit_value: "Comment".to_string(),
            owner,
            repository,
        },
    }))
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
        user.email.as_ref().expect("local user has email"),
    )?;

    pull_request.merged(&db).await?;

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

    git_repository.merge(
        &pull_request.base,
        &pull_request.compare,
        &user.username,
        user.email.as_ref().expect("local user has email"),
    )?;

    pull_request.merged(&db).await?;

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
