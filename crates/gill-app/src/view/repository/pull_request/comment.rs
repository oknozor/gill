use crate::domain::repository::Repository;
use crate::error::{AppError, AppResult};
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use axum::extract::Path;
use axum::response::Redirect;
use axum::{Extension, Form};
use gill_authorize_derive::authorized;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize, Debug)]
pub struct CommentPullRequestForm {
    pub content: String,
}

#[authorized]
pub async fn comment(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, pull_request_number)): Path<(String, String, i32)>,
    Form(input): Form<CommentPullRequestForm>,
) -> AppResult<Redirect> {
    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .get_pull_request(pull_request_number, &db)
        .await?
        .comment(&input.content, user.id, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/pulls/{pull_request_number}"
    )))
}
