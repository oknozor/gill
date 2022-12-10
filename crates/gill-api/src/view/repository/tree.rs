use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::repository::BranchDto;
use crate::view::{get_connected_user_username, HtmlTemplate};
use anyhow::anyhow;
use askama::Template;
use axum::extract::Path;
use axum::{Extension, TypedHeader};
use gill_db::user::User;
use gill_git::traversal::TreeMap;
use headers::ContentType;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/tree.html")]
pub struct GitTreeTemplate {
    tree: TreeMap,
    readme: Option<String>,
    branches: Vec<BranchDto>,
    current_branch: String,
    user: Option<String>,
}

/// Returns a tree with for a given owner, repository and a branch
pub async fn tree(
    user: Option<Oauth2User>,
    Path((owner, repository, branch)): Path<(String, String, String)>,
    Path(path): Path<Vec<String>>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let tree = path[3..].join("/");
    let tree_path = if tree.is_empty() {
        None
    } else {
        Some(tree.as_str())
    };

    imp::get_tree(
        owner,
        repository,
        tree_path,
        branch,
        connected_username,
        &db,
    )
    .await
}

/// Returns the root of a tree with for a given owner and repository
/// using the given branch
pub async fn tree_root(
    user: Option<Oauth2User>,
    Path((owner, repository, branch)): Path<(String, String, String)>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    imp::get_tree_root(&owner, &repository, branch, connected_username, &db).await
}

/// Returns the root of a tree with for a given owner and repository
/// using the default branch
pub async fn root(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
    let connected_username = get_connected_user_username(&db, user).await;
    let user = User::by_user_name(&owner, &db).await?;
    let repo = user.get_local_repository_by_name(&repository, &db).await?;

    let branch = repo
        .get_default_branch(&db)
        .await
        .ok_or_else(|| anyhow!("No default branch"))?;

    imp::get_tree_root(&owner, &repository, branch.name, connected_username, &db).await
}

mod imp {
    use super::GitTreeTemplate;
    use crate::error::AppError;
    use crate::view::repository::tree::BranchDto;
    use crate::view::HtmlTemplate;

    use gill_db::user::User;

    use gill_git::traversal::{get_tree_for_path, TreeMap};
    use pulldown_cmark::{html, Options, Parser};
    use sqlx::PgPool;
    use std::env;
    use std::path::PathBuf;

    pub(crate) async fn get_tree_root(
        owner: &str,
        repository: &str,
        current_branch: String,
        connected_username: Option<String>,
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
        let repo = user.get_local_repository_by_name(repository, db).await?;
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
            user: connected_username,
        };

        Ok(HtmlTemplate(template))
    }

    pub(crate) async fn get_tree(
        owner: String,
        repository: String,
        tree_path: Option<&str>,
        current_branch: String,
        connected_username: Option<String>,
        db: &PgPool,
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
        let repo = user.get_local_repository_by_name(&repository, db).await?;
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
            user: connected_username,
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
