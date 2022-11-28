use crate::error::AppError;
use crate::syntax::highlight_blob;
use crate::view::repository::blob::BlobDto::{Highlighted, PlainText};
use crate::view::repository::BranchDto;
use crate::view::{repository, HtmlTemplate};
use askama::Template;
use axum::extract::{Path, State};
use axum::Extension;
use ruisseau_db::user::User;
use ruisseau_git::traversal::get_tree_for_path;
use sqlx::PgPool;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::{env, fmt};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

#[derive(Template)]
#[template(path = "repository/blob.html")]
pub struct GitBLobTemplate {
    blob: BlobDto,
    language: Option<String>,
    branches: Vec<BranchDto>,
    current_branch: String,
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
    Path((owner, repository, current_branch)): Path<(String, String, String)>,
    Path(path): Path<Vec<String>>,
    Extension(db): Extension<PgPool>,
    State(syntax_set): State<SyntaxSet>,
    State(theme): State<Theme>,
) -> Result<HtmlTemplate<GitBLobTemplate>, AppError> {
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

    let user = User::by_user_name(&owner, &db).await.unwrap();
    /// TODO: factorize with tree
    let repository = user.get_repository_by_name(&repository, &db).await?;
    let branches = repository.list_branches(20, 0, &db).await?;
    let branches = branches
        .into_iter()
        .map(|branch| {
            let is_current = branch.name == current_branch;

            BranchDto {
                name: branch.name,
                is_default: branch.is_default,
                is_current,
            }
        })
        .collect();

    let template = GitBLobTemplate {
        blob,
        language,
        branches,
        current_branch,
    };
    Ok(HtmlTemplate(template))
}

pub fn get_blob_language(blob_name: &str) -> Option<String> {
    blob_name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_string())
}
