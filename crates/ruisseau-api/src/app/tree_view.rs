use axum::extract::Path;
use ruisseau_git::traversal::{traverse, TreeMap};
use askama::Template;
use crate::app::HtmlTemplate;
use crate::error::AppError;

#[derive(Template)]
#[template(path = "tree.html")]
pub struct GitTreeTemplate {
    tree: TreeMap,
}

pub async fn tree(Path((owner, repository, tree)): Path<(String, String, String)>) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    let repo_path = format!("{owner}/{repository}");
    let tree = traverse(repo_path, Some(&tree))?;
    let template = GitTreeTemplate { tree };
    Ok(HtmlTemplate(template))
}