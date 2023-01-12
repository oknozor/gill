use crate::domain::repository::Repository;
use crate::error::{AppError, AppResult};
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use axum::extract::Path;
use axum::response::Redirect;
use axum::Extension;
use gill_authorize_derive::authorized;
use sqlx::PgPool;

#[authorized]
pub async fn close(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, issue_number)): Path<(String, String, i32)>,
) -> AppResult<Redirect> {
    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .close_issue(issue_number, user.activity_pub_id, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/issues/{issue_number}"
    )))
}
