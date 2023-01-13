use crate::domain::repository::stats::RepositoryStats;
use crate::error::AppResult;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;

use crate::view::HtmlTemplate;

use askama::Template;
use axum::extract::Path;
use axum::Extension;

use crate::domain::issue::digest::IssueDigest;
use crate::domain::issue::IssueState;
use crate::domain::repository::Repository;
use crate::view::repository::Tab;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/issues/list.html")]
pub struct IssuesTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    issues: Option<Vec<IssueDigest>>,
    stats: RepositoryStats,
    current_branch: Option<String>,
    tab: Tab,
}

pub async fn list_view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> AppResult<HtmlTemplate<IssuesTemplate>> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let issues = repo.list_issues(&db).await?;
    let pull_requests = (!issues.is_empty()).then_some(issues);
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);

    Ok(HtmlTemplate(IssuesTemplate {
        user: connected_username,
        owner,
        repository,
        issues: pull_requests,
        stats,
        current_branch,
        tab: Tab::Issues,
    }))
}
