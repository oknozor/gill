use crate::domain::repository::stats::RepositoryStats;
use crate::error::{AppError, AppResult};
use crate::oauth::Oauth2User;
use crate::view::component::MarkdownPreviewForm;
use crate::view::HtmlTemplate;
use crate::{get_connected_user, get_connected_user_username};

use crate::domain::pull_request::comment::PullRequestComment;
use crate::domain::pull_request::{PullRequest, PullRequestState};
use crate::domain::repository::Repository;
use askama::Template;
use axum::extract::Path;
use axum::response::Redirect;
use axum::Extension;

use crate::view::repository::Tab;
use gill_authorize_derive::authorized;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/pulls/pull.html")]
pub struct PullRequestTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_request: PullRequest,
    stats: RepositoryStats,
    current_branch: Option<String>,
    comments: Vec<PullRequestComment>,
    markdown_preview_form: MarkdownPreviewForm,
    tab: Tab,
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
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);

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
        current_branch,
        comments,
        markdown_preview_form: MarkdownPreviewForm {
            action_href,
            submit_value: "Comment".to_string(),
            owner,
            repository,
        },
        tab: Tab::PullRequests,
    }))
}

#[authorized]
pub async fn rebase(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> AppResult<Redirect> {
    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .rebase(&user, &owner, pull_request_number, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}

#[authorized]
pub async fn merge(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<Redirect, AppError> {
    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .merge(&user, &owner, pull_request_number, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}

#[authorized]
pub async fn close(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<Redirect, AppError> {
    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .close_pull_request(&user, pull_request_number, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}
