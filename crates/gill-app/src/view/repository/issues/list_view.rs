use crate::domain::repository::RepositoryStats;
use crate::error::AppError;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;

use crate::view::HtmlTemplate;

use askama::Template;
use axum::extract::Path;
use axum::Extension;

use gill_db::repository::issue::IssueDigest;
use gill_db::repository::issue::IssueState;
use gill_db::repository::Repository;
use sqlx::PgPool;
use std::cmp::Ordering;

#[derive(Template, Debug)]
#[template(path = "repository/issues/list.html")]
pub struct IssuesTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    issues: Option<Vec<IssueDigest>>,
    stats: RepositoryStats,
    current_branch: Option<String>,
}

pub async fn list_view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<HtmlTemplate<IssuesTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let mut issues = repo.list_issues(&db).await?;
    issues.sort_by(|issues, other| match (&issues.state, &other.state) {
        (IssueState::Open, IssueState::Closed) => Ordering::Less,
        (_, _) => issues.number.cmp(&other.number),
    });

    let pull_requests = (!issues.is_empty()).then_some(issues);
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);

    Ok(HtmlTemplate(IssuesTemplate {
        user: connected_username,
        owner,
        repository,
        issues: pull_requests,
        stats,
        current_branch,
    }))
}
