use crate::domain::repository::RepositoryStats;
use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::component::MarkdownPreviewForm;

use crate::get_connected_user_username;

use crate::view::HtmlTemplate;

use askama::Template;
use axum::extract::Path;

use axum::Extension;
use gill_db::repository::Repository;

use gill_db::repository::issue::comment::IssueCommentDigest;
use gill_db::repository::issue::{IssueDigest, IssueState};
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/issues/issue.html")]
pub struct IssueTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    issue: IssueDigest,
    stats: RepositoryStats,
    current_branch: Option<String>,
    comments: Vec<IssueCommentDigest>,
    markdown_preview_form: MarkdownPreviewForm,
}

pub async fn view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, issue_number)): Path<(String, String, i32)>,
) -> Result<HtmlTemplate<IssueTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let issue = repo.get_issue_digest(issue_number, &db).await?;
    let comments = issue.get_comments(&db).await?;
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);

    let action_href = format!("/{owner}/{repository}/issues/{issue_number}/comment");
    Ok(HtmlTemplate(IssueTemplate {
        user: connected_username,
        owner: owner.clone(),
        repository: repository.clone(),
        issue,
        stats,
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
