use std::env;
use std::path::PathBuf;
use axum::extract::Path;
use ruisseau_git::traversal::{traverse, TreeMap};
use askama::Template;
use pulldown_cmark::{html, Options, Parser};
use crate::app::HtmlTemplate;
use crate::error::AppError;
use crate::SETTINGS;

#[derive(Template)]
#[template(path = "tree.html")]
pub struct GitTreeTemplate {
    tree: TreeMap,
    readme: Option<String>,
    branches: Vec<String>,
    current_branch: String,
}


pub async fn tree(
    Path((owner, repository, branch)): Path<(String, String, String)>,
    Path(path): Path<Vec<String>>) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    let tree = path[3..].join("/");
    let tree = if tree.is_empty() {
        None
    } else {
        Some(tree.as_str())
    };

    let repo_path = format!("{owner}/{repository}.git");

    if !PathBuf::from(&repo_path).exists() {
        let current_dir = env::current_dir().unwrap();
        let current_dir = current_dir.to_string_lossy();
        tracing::error!("Repository not found '{current_dir}/{repo_path}'")
    }

    let tree = traverse(&repo_path, Some(&branch), tree)?;
    let readme = get_readme(&tree, &repo_path);
    let branches = ruisseau_git::repository::list_branch(&SETTINGS.repo_dir, &owner, &repository)?;
    let template = GitTreeTemplate { tree, readme, branches, current_branch: branch };
    Ok(HtmlTemplate(template))
}

pub async fn tree_root(Path((owner, repository, branch)): Path<(String, String, String)>) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    let repo_path = format!("{owner}/{repository}.git");

    if !PathBuf::from(&repo_path).exists() {
        let current_dir = env::current_dir().unwrap();
        let current_dir = current_dir.to_string_lossy();
        tracing::error!("Repository not found '{current_dir}/{repo_path}'")
    }

    let tree = traverse(&repo_path, Some(&branch), None)?;
    let readme = get_readme(&tree, &repo_path);
    let branches = ruisseau_git::repository::list_branch(&SETTINGS.repo_dir, &owner, &repository)?;
    let template = GitTreeTemplate { tree, readme, branches, current_branch: branch };
    Ok(HtmlTemplate(template))
}

fn get_readme(tree: &TreeMap, repo_path: &str) -> Option<String> {
    tree.blobs.iter()
        .find(|blob| &blob.filename.to_string() == "README.md")
        .and_then(|blob| blob.content(repo_path).ok())
        .map(|readme| {
            let parser = Parser::new_ext(&readme, Options::empty());
            let mut html = String::new();
            html::push_html(&mut html, parser);
            html
        })
}