use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::syntax::highlight::highlight_blob;
use crate::view::repository::blob::BlobDto::{Highlighted, PlainText};
use crate::view::repository::{get_repository_branches, BranchDto};
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::{Path, State};
use axum::Extension;

use crate::get_connected_user_username;
use gill_db::repository::RepositoryLight;
use gill_git::repository::traversal::get_tree_for_path;
use sqlx::PgPool;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::{env, fmt};
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;

#[derive(Template)]
#[template(path = "repository/blob.html")]
pub struct GitBLobTemplate {
    repository: String,
    owner: String,
    watch_count: u32,
    fork_count: u32,
    star_count: u32,
    blob: BlobDto,
    language: Option<String>,
    branches: Vec<BranchDto>,
    current_branch: String,
    user: Option<String>,
}

#[derive(Debug)]
enum BlobDto {
    Highlighted(String),
    PlainText(String),
}

impl fmt::Display for BlobDto {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self:?}")
    }
}

pub async fn blob(
    user: Option<Oauth2User>,
    Path((owner, repository, current_branch)): Path<(String, String, String)>,
    Path(path): Path<Vec<String>>,
    Extension(db): Extension<PgPool>,
    State(syntax_set): State<SyntaxSet>,
    State(theme): State<Theme>,
) -> Result<HtmlTemplate<GitBLobTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let path = path.last().unwrap();
    let (tree, blob_name) = match path.rsplit_once('/') {
        None => (None, path.as_str()),
        Some((tree, blob_name)) => {
            if !tree.is_empty() {
                (Some(tree), blob_name)
            } else {
                (None, blob_name)
            }
        }
    };

    let repo_path = format!("{owner}/{repository}.git");
    if !PathBuf::from(&repo_path).exists() {
        let current_dir = env::current_dir().unwrap();
        let current_dir = current_dir.to_string_lossy();
        tracing::error!("Repository not found '{current_dir}/{repo_path}'")
    }

    let tree = get_tree_for_path(&repo_path, Some(&current_branch), tree)?;
    let blob = tree
        .blobs
        .iter()
        .find(|blob| blob.filename == blob_name)
        .unwrap();
    let blob = blob.content(&repo_path)?;
    let language = get_blob_language(blob_name);
    let blob = language
        .as_ref()
        .and_then(|language| highlight_blob(&blob, language, syntax_set, &theme).ok())
        .map(Highlighted)
        .unwrap_or(PlainText(blob));

    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;
    let stats = RepositoryLight::stats_by_namespace(&owner, &repository, &db).await?;

    let template = GitBLobTemplate {
        repository,
        owner,
        watch_count: stats.watch_count.unwrap_or(0) as u32,
        fork_count: stats.fork_count.unwrap_or(0) as u32,
        star_count: stats.star_count.unwrap_or(0) as u32,
        blob,
        language,
        branches,
        current_branch,
        user: connected_username,
    };
    Ok(HtmlTemplate(template))
}

pub fn get_blob_language(blob_name: &str) -> Option<String> {
    blob_name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_string())
}
