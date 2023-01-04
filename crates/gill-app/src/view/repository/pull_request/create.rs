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
pub struct CreatePullRequestForm {
    pub title: String,
    pub description: String,
    pub base: String,
    pub compare: String,
}

pub async fn create(
    connected_user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
    Form(input): Form<CreatePullRequestForm>,
) -> Result<Redirect, AppError> {
    let Some(user) = get_connected_user(&db, connected_user).await else {
        return Err(AppError::Unauthorized);
    };

    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    repo.create_pull_request(
        user.id,
        &input.title,
        Some(&input.description.escape_default().to_string()),
        &input.base,
        &input.compare,
        &db,
    )
    .await?;
    Ok(Redirect::to(&format!("/{owner}/{repository}/pulls")))
}
