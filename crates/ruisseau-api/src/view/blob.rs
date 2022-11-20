use crate::error::AppError;
use crate::view::HtmlTemplate;
use crate::SETTINGS;
use askama::Template;
use axum::extract::Path;
use ruisseau_git::traversal::traverse;
use std::env;
use std::path::PathBuf;

#[derive(Template)]
#[template(path = "blob.html")]
pub struct GitBLobTemplate {
    blob: String,
    language: Option<String>,
    branches: Vec<String>,
    current_branch: String,
}

pub async fn blob(
    Path((owner, repository, branch)): Path<(String, String, String)>,
    Path(path): Path<Vec<String>>,
) -> Result<HtmlTemplate<GitBLobTemplate>, AppError> {
    let path = path.last().unwrap();
    let (tree, blob_name) = match path.rsplit_once("/") {
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

    let tree = traverse(&repo_path, Some(&branch), tree)?;
    let blob = tree
        .blobs
        .iter()
        .find(|blob| &blob.filename.to_string() == &blob_name)
        .unwrap();
    let blob = blob.content(&repo_path)?;
    let language = get_blob_language(&blob_name);
    let branches = ruisseau_git::repository::list_branch(&SETTINGS.repo_dir, &owner, &repository)?;
    println!("{:?}", branches);
    let template = GitBLobTemplate {
        blob,
        language,
        branches,
        current_branch: branch,
    };
    Ok(HtmlTemplate(template))
}

pub fn get_blob_language(blob_name: &str) -> Option<String> {
    blob_name
        .rsplit_once(".")
        .map(|(_, extension)| extension.to_string())
}
