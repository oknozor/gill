use crate::error::AppResult;
use crate::oauth::Oauth2User;
use crate::view::repository::{get_repository_branches, tree_and_blob_from_query, BranchDto};
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::Path;
use axum::Extension;
use gill_syntax::highlight::highlight_blob;

use crate::get_connected_user_username;

use gill_git::traversal::BlobMime;
use gill_git::GitRepository;
use sqlx::PgPool;
use std::fmt::Formatter;

use crate::domain::repository::stats::RepositoryStats;
use base64::engine::general_purpose;
use base64::Engine;
use std::fmt;

// Needed in template
use crate::view::repository::blob::BlobDto::*;

#[derive(Template)]
#[template(path = "repository/tree/blob.html")]
pub struct GitBLobTemplate {
    repository: String,
    owner: String,
    stats: RepositoryStats,
    blob: BlobDto,
    branches: Vec<BranchDto>,
    current_branch: Option<String>,
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
) -> AppResult<HtmlTemplate<GitBLobTemplate>> {
    let connected_username = get_connected_user_username(&db, user).await;
    let path = path.last().unwrap();
    let (tree, blob_name) = tree_and_blob_from_query(path);

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
                    highlight_blob(&blob, language)
                        .ok()
                        .map(|hl| (language, hl))
                })
                .map(|(language, content)| Highlighted {
                    content,
                    language: language.to_string(),
                })
                .unwrap_or(PlainText(blob))
        }
        BlobMime::Image => Image(general_purpose::STANDARD.encode(repo.blob_bytes(blob)?)),
        BlobMime::Application => Binary {
            content: general_purpose::STANDARD.encode(repo.blob_bytes(blob)?),
            filename: blob.filename.clone(),
        },
    };

    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;

    let template = GitBLobTemplate {
        repository,
        owner,
        stats,
        blob,
        branches,
        current_branch: Some(current_branch),
        user: connected_username,
    };
    Ok(HtmlTemplate(template))
}

pub fn get_blob_language(blob_name: &str) -> Option<String> {
    blob_name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_string())
}
