use crate::domain::pull_request::PullRequest;
use crate::domain::pull_request::PullRequestState;
use crate::domain::repository::stats::RepositoryStats;
use crate::domain::repository::Repository;
use crate::error::AppError;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;
use crate::view::repository::Tab;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::Path;
use axum::Extension;
use gill_syntax::diff::diff2html;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/pulls/diff.html")]
pub struct PullRequestDiffTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    pull_request: PullRequest,
    stats: RepositoryStats,
    current_branch: Option<String>,
    diff: String,
    tab: Tab,
}

pub async fn diff(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
) -> Result<HtmlTemplate<PullRequestDiffTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let pull_request = repo.get_pull_request(pull_request_number, &db).await?;
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);
    let diff = pull_request.get_diff(&owner, &repository)?;
    let diff = diff2html(&diff)?;
    Ok(HtmlTemplate(PullRequestDiffTemplate {
        user: connected_username,
        owner: owner.clone(),
        repository: repository.clone(),
        pull_request,
        stats,
        current_branch,
        diff,
        tab: Tab::PullRequests,
    }))
}
