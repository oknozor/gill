use crate::domain::repository::RepositoryStats;
use crate::error::AppError;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;

use crate::view::HtmlTemplate;

use askama::Template;
use axum::extract::Path;
use axum::Extension;
use gill_db::repository::pull_request::{PullRequest, PullRequestState};
use gill_db::repository::Repository;
use sqlx::PgPool;
use std::cmp::Ordering;

#[derive(Template, Debug)]
#[template(path = "repository/pulls/list.html")]
pub struct PullRequestsTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_requests: Option<Vec<PullRequest>>,
    stats: RepositoryStats,
    current_branch: Option<String>,
}

pub async fn list_view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<HtmlTemplate<PullRequestsTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let mut pull_requests = repo.list_pull_requests(&db).await?;
    pull_requests.sort_by(|pr, other| match (&pr.state, &other.state) {
        (PullRequestState::Open, PullRequestState::Closed)
        | (PullRequestState::Open, PullRequestState::Merged) => Ordering::Less,
        (_, _) => pr.number.cmp(&other.number),
    });

    let pull_requests = (!pull_requests.is_empty()).then_some(pull_requests);
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);

    Ok(HtmlTemplate(PullRequestsTemplate {
        user: connected_username,
        owner,
        repository,
        pull_requests,
        stats,
        current_branch,
    }))
}
