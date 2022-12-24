use crate::error::AppError;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::{Path, Query};
use axum::Extension;
use gill_git::repository::GitRepository;
use gill_syntax::diff::diff2html;
use serde::Deserialize;
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

pub async fn view(
    user: Option<Oauth2User>,
    Path((owner, repository)): Path<(String, String)>,
    Query(diff): Query<DiffQuery>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<GitDiffTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let repo = GitRepository::open(&owner, &repository)?;
    let diff = repo.diff(&diff.from, &diff.to)?;
    let diff = diff2html(&diff)?;

    Ok(HtmlTemplate(GitDiffTemplate {
        repository,
        owner,
        diff,
        user: connected_username,
    }))
}

pub async fn get_diff(
    Path((owner, repository)): Path<(String, String)>,
    Query(diff): Query<DiffQuery>,
) -> Result<String, AppError> {
    let repo = GitRepository::open(&owner, &repository)?;
    let diff = repo.diff(&diff.from, &diff.to)?;
    let diff = diff2html(&diff)?;

    Ok(diff)
}
