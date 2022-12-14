use crate::error::AppError;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;
use crate::syntax::diff::HtmlDiff;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::Extension;
use serde::Deserialize;
use sqlx::PgPool;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;

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
    content: String,
    user: Option<String>,
}

pub async fn diff(
    user: Option<Oauth2User>,
    Path((owner, repository)): Path<(String, String)>,
    Query(diff): Query<DiffQuery>,
    State(syntax_set): State<SyntaxSet>,
    State(theme): State<Theme>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<GitDiffTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let repo = gill_git::repository::open(&owner, &repository)?;
    let mut writer = HtmlDiff::new(syntax_set, theme);
    gill_git::repository::diff::diff(&repo, &diff.from, &diff.to, &mut writer)?;

    Ok(HtmlTemplate(GitDiffTemplate {
        repository,
        owner,
        content: writer.get_html()?,
        user: connected_username,
    }))
}
