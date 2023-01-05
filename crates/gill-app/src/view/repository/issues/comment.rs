use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use axum::extract::Path;
use axum::response::Redirect;
use axum::{Extension, Form};
use gill_db::repository::Repository;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize, Debug)]
pub struct IssueCommentForm {
    pub comment: String,
}

pub async fn comment(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, issue_number)): Path<(String, String, i32)>,
    Form(input): Form<IssueCommentForm>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    let comment = input.comment.escape_default().to_string();
    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .get_issue(issue_number, &db)
        .await?
        .comment(&comment, user.id, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/issues/{issue_number}"
    )))
}
