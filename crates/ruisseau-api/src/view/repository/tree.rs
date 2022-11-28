use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::repository::BranchDto;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::Path;
use axum::Extension;
use eyre::eyre;
use ruisseau_db::user::User;
use ruisseau_git::traversal::TreeMap;
use sqlx::PgPool;
use std::fmt::Formatter;

#[derive(Template, Debug)]
#[template(path = "repository/tree.html")]
pub struct GitTreeTemplate {
    tree: TreeMap,
    readme: Option<String>,
    branches: Vec<BranchDto>,
    current_branch: String,
}

/// Returns a tree with for a given owner, repository and a branch
pub async fn tree(
    Path((owner, repository, branch)): Path<(String, String, String)>,
    Path(path): Path<Vec<String>>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    let tree = path[3..].join("/");
    let tree_path = if tree.is_empty() {
        None
    } else {
        Some(tree.as_str())
    };

    imp::get_tree(owner, repository, tree_path, &db, branch).await
}

/// Returns the root of a tree with for a given owner and repository
/// using the given branch
pub async fn tree_root(
    Path((owner, repository, branch)): Path<(String, String, String)>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    imp::get_tree_root(&owner, &repository, branch, &db).await
}

/// Returns the root of a tree with for a given owner and repository
/// using the default branch
pub async fn root(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    /// TODO: Add special view for (owner, maintainer, dev)
    let user = User::by_user_name(&owner, &db).await?;
    let repo = user.get_repository_by_name(&repository, &db).await?;

    let branch = repo
        .get_default_branch(&db)
        .await
        .ok_or(eyre!("No default branch"))?;

    imp::get_tree_root(&owner, &repository, branch.name, &db).await
}

mod imp {
    use super::GitTreeTemplate;
    use crate::error::AppError;
    use crate::view::repository::tree::BranchDto;
    use crate::view::HtmlTemplate;
    use crate::SETTINGS;
    use pulldown_cmark::{html, Options, Parser};
    use ruisseau_db::repository::Repository;
    use ruisseau_db::user::User;
    use ruisseau_git::repository;
    use ruisseau_git::traversal::{get_tree_for_path, TreeMap};
    use sqlx::PgPool;
    use std::env;
    use std::path::PathBuf;

    pub(crate) async fn get_tree_root(
        owner: &str,
        repository: &str,
        current_branch: String,
        db: &PgPool,
    ) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
        let repo_path = format!("{owner}/{repository}.git");

        if !PathBuf::from(&repo_path).exists() {
            let current_dir = env::current_dir().unwrap();
            let current_dir = current_dir.to_string_lossy();
            tracing::error!("Repository not found '{current_dir}/{repo_path}'")
        }
        let tree = get_tree_for_path(&repo_path, Some(&current_branch), None)?;
        let readme = get_readme(&tree, &repo_path);
        let user = User::by_user_name(owner, db).await?;
        let repo = user.get_repository_by_name(repository, db).await?;
        let branches = repo.list_branches(0, 20, db).await?;
        let branches = branches
            .into_iter()
            .map(|branch| BranchDto {
                name: branch.name,
                is_default: branch.is_default,
                is_current: false,
            })
            .collect();

        let template = GitTreeTemplate {
            tree,
            readme,
            branches,
            current_branch,
        };

        Ok(HtmlTemplate(template))
    }

    pub(crate) async fn get_tree(
        owner: String,
        repository: String,
        tree_path: Option<&str>,
        db: &PgPool,
        current_branch: String,
    ) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
        let repo_path = format!("{owner}/{repository}.git");
        if !PathBuf::from(&repo_path).exists() {
            let current_dir = env::current_dir().unwrap();
            let current_dir = current_dir.to_string_lossy();
            tracing::error!("Repository not found '{current_dir}/{repo_path}'")
        };

        let tree = get_tree_for_path(&repo_path, Some(&current_branch), tree_path)?;
        let readme = get_readme(&tree, &repo_path);
        let user = User::by_user_name(&owner, db).await?;
        let repo = user.get_repository_by_name(&repository, db).await?;
        let branches = repo.list_branches(0, 20, db).await?;
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

        let template = GitTreeTemplate {
            tree,
            readme,
            branches,
            current_branch,
        };

        Ok(HtmlTemplate(template))
    }

    pub fn get_readme(tree: &TreeMap, repo_path: &str) -> Option<String> {
        tree.blobs
            .iter()
            .find(|blob| &blob.filename.to_string() == "README.md")
            .and_then(|blob| blob.content(repo_path).ok())
            .map(|readme| {
                let parser = Parser::new_ext(&readme, Options::empty());
                let mut html = String::new();
                html::push_html(&mut html, parser);
                html
            })
    }
}
