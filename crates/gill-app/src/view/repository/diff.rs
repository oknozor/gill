use crate::error::AppError;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::{Path, Query};
use axum::Extension;
use serde::{Deserialize};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct DiffQuery {
    from: String,
    to: String,
}

#[derive(Template)]
#[template(path = "repository/diff.html")]
pub struct GitDiffTemplate {
    repository: String,
    owner: String,
    diff: String,
    user: Option<String>,
}

pub async fn diff<'a>(
    user: Option<Oauth2User>,
    Path((owner, repository)): Path<(String, String)>,
    Query(diff): Query<DiffQuery>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<GitDiffTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let repo = gill_git::repository::open(&owner, &repository)?;
    let diff = gill_git::repository::diff::diff(&repo, &diff.from, &diff.to)?;
    let diff = diff.replace('`', "\'");
    let diff = diff.replace('$', "\\$");
    Ok(HtmlTemplate(GitDiffTemplate {
        repository,
        owner,
        diff,
        user: connected_username,
    }))
}
