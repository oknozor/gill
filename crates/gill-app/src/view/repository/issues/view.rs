use crate::domain::repository::RepositoryStats;
use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::component::MarkdownPreviewForm;

use crate::get_connected_user_username;
use crate::view::repository::{get_repository_branches, BranchDto};
use crate::view::HtmlTemplate;
use anyhow::anyhow;
use askama::Template;
use axum::extract::Path;

use axum::Extension;
use gill_db::repository::issue::{Issue, IssueComment, IssueState};

use gill_db::repository::Repository;

use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/issues/issue.html")]
pub struct IssueTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    issue: Issue,
    stats: RepositoryStats,
    branches: Vec<BranchDto>,
    current_branch: String,
    comments: Vec<IssueComment>,
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
    let issue = repo.get_issue(issue_number, &db).await?;
    let comments = issue.get_comments(&db).await?;
    let current_branch = repo
        .get_default_branch(&db)
        .await
        .ok_or_else(|| anyhow!("No default branch"))?;

    let current_branch = current_branch.name;
    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;
    let action_href = format!("/{owner}/{repository}/issues/{issue_number}/comment");
    Ok(HtmlTemplate(IssueTemplate {
        user: connected_username,
        owner: owner.clone(),
        repository: repository.clone(),
        issue,
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
