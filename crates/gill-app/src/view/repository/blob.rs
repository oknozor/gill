use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::syntax::highlight::highlight_blob;
use crate::view::repository::{get_repository_branches, BranchDto};
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::{Path, State};
use axum::Extension;

use crate::get_connected_user_username;
use gill_db::repository::RepositoryLight;
use gill_git::repository::traversal::BlobMime;
use gill_git::repository::GitRepository;
use sqlx::PgPool;
use std::fmt::Formatter;

use std::fmt;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;

// Needed in template
use crate::view::repository::blob::BlobDto::*;

#[derive(Template)]
#[template(path = "repository/blob.html")]
pub struct GitBLobTemplate {
    repository: String,
    owner: String,
    watch_count: u32,
    fork_count: u32,
    star_count: u32,
    blob: BlobDto,
    branches: Vec<BranchDto>,
    current_branch: String,
    user: Option<String>,
}

#[derive(Debug)]
enum BlobDto {
    Highlighted { content: String, language: String },
    PlainText(String),
    Image(String),
    Binary { content: String, filename: String },
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

    let repo = GitRepository::open(&owner, &repository)?;
    let tree = repo.get_tree_for_path(Some(&current_branch), tree)?;
    let blob = tree
        .blobs
        .iter()
        .find(|blob| blob.filename() == blob_name)
        .unwrap();
    let blob = match repo.blob_mime(blob) {
        BlobMime::Text => {
            let blob = repo.blob_str(blob)?;
            let language = get_blob_language(blob_name);
            language
                .as_ref()
                .and_then(|language| {
                    highlight_blob(&blob, language, syntax_set, &theme)
                        .ok()
                        .map(|hl| (language, hl))
                })
                .map(|(language, content)| Highlighted {
                    content,
                    language: language.to_string(),
                })
                .unwrap_or(PlainText(blob))
        }
        BlobMime::Image => Image(base64::encode(repo.blob_bytes(blob)?)),
        BlobMime::Application => Binary {
            content: base64::encode(repo.blob_bytes(blob)?),
            filename: blob.filename.clone(),
        },
    };

    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;
    let stats = RepositoryLight::stats_by_namespace(&owner, &repository, &db).await?;

    let template = GitBLobTemplate {
        repository,
        owner,
        watch_count: stats.watch_count.unwrap_or(0) as u32,
        fork_count: stats.fork_count.unwrap_or(0) as u32,
        star_count: stats.star_count.unwrap_or(0) as u32,
        blob,
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
