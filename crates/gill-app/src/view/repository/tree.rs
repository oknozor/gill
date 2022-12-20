use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::repository::BranchDto;
use crate::view::HtmlTemplate;
use anyhow::anyhow;
use askama::Template;
use axum::extract::Path;
use axum::Extension;
use gill_db::user::User;
use gill_git::repository::traversal::TreeMap;

use crate::get_connected_user_username;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/tree.html")]
pub struct GitTreeTemplate {
    repository: String,
    owner: String,
    watch_count: u32,
    fork_count: u32,
    star_count: u32,
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

    use crate::view::repository::get_repository_branches;
    use gill_db::repository::RepositoryLight;
    use gill_git::repository::traversal::TreeMap;
    use gill_git::repository::GitRepository;
    use pulldown_cmark::{html, Options, Parser};
    use sqlx::PgPool;

    pub(crate) async fn get_tree_root(
        owner: &str,
        repository: &str,
        current_branch: String,
        connected_username: Option<String>,
        db: &PgPool,
    ) -> Result<HtmlTemplate<GitTreeTemplate>, AppError> {
        let repo = GitRepository::open(owner, repository)?;
        let tree = repo.get_tree_for_path(Some(&current_branch), None)?;
        let readme = get_readme(&tree, &repo);
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

        let stats = RepositoryLight::stats_by_namespace(owner, repository, db).await?;

        let template = GitTreeTemplate {
            repository: repository.to_string(),
            owner: owner.to_string(),
            watch_count: stats.watch_count.unwrap_or(0) as u32,
            fork_count: stats.fork_count.unwrap_or(0) as u32,
            star_count: stats.star_count.unwrap_or(0) as u32,
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
        let repo = GitRepository::open(&owner, &repository)?;
        let tree = repo.get_tree_for_path(Some(&current_branch), tree_path)?;
        let readme = get_readme(&tree, &repo);
        let branches = get_repository_branches(&owner, &repository, &current_branch, db).await?;
        let stats = RepositoryLight::stats_by_namespace(&owner, &repository, db).await?;

        let template = GitTreeTemplate {
            repository,
            owner,
            watch_count: stats.watch_count.unwrap_or(0) as u32,
            fork_count: stats.fork_count.unwrap_or(0) as u32,
            star_count: stats.star_count.unwrap_or(0) as u32,
            tree,
            readme,
            branches,
            current_branch,
            user: connected_username,
        };

        Ok(HtmlTemplate(template))
    }

    pub fn get_readme(tree: &TreeMap, repo: &GitRepository) -> Option<String> {
        tree.blobs
            .iter()
            .find(|blob| &blob.filename() == "README.md")
            .and_then(|blob| repo.blob_str(blob).ok())
            .map(|readme| {
                let parser = Parser::new_ext(&readme, Options::empty());
                let mut html = String::new();
                html::push_html(&mut html, parser);
                html
            })
    }
}
