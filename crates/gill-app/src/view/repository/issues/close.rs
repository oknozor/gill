use crate::domain::repository::Repository;
use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use axum::extract::Path;
use axum::response::Redirect;
use axum::Extension;
use sqlx::PgPool;

pub async fn close(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, issue_number)): Path<(String, String, i32)>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .close_issue(issue_number, user.activity_pub_id, &db)
        .await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/issues/{issue_number}"
    )))
}
