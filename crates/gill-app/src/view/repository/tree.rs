use crate::error::AppResult;
use crate::oauth::Oauth2User;
use crate::view::repository::BranchDto;
use crate::view::{DynHtmlTemplate, HtmlTemplate};
use anyhow::anyhow;
use askama::{DynTemplate, Template};
use axum::extract::Path;

use axum::Extension;

use gill_git::traversal::{BlobInfo, TreeEntry, TreeInfo};

use crate::domain::repository::stats::RepositoryStats;
use crate::get_connected_user_username;

use crate::domain::repository::Repository;
use sqlx::PgPool;

#[derive(Debug)]
struct TreeDto {
    pub filename: String,
    pub blobs: Vec<BlobDto>,
    pub trees: Vec<TreeEntryDto>,
}

#[derive(Debug)]
struct BlobDto {
    filename: String,
    commit_summary: String,
    commit_sha: String,
}

#[derive(Debug)]
struct TreeEntryDto {
    filename: String,
    commit_summary: String,
    commit_sha: String,
}

impl From<BlobInfo> for BlobDto {
    fn from(blob: BlobInfo) -> Self {
        BlobDto {
            filename: blob.filename,
            commit_summary: blob.commit.summary,
            commit_sha: blob.commit.id,
        }
    }
}

impl From<TreeInfo> for TreeEntryDto {
    fn from(entry: TreeInfo) -> Self {
        TreeEntryDto {
            filename: entry.name,
            commit_summary: entry.commit.summary,
            commit_sha: entry.commit.id,
        }
    }
}

impl From<TreeEntry> for TreeDto {
    fn from(tree: TreeEntry) -> Self {
        let mut trees: Vec<_> = tree.trees.into_iter().map(TreeEntryDto::from).collect();

        trees.sort_by(|a, b| a.filename.cmp(&b.filename));

        let mut blobs: Vec<BlobDto> = tree.blobs.into_iter().map(BlobDto::from).collect();

        blobs.sort_by(|a, b| a.filename.cmp(&b.filename));

        Self {
            filename: tree.filename,
            blobs,
            trees,
        }
    }
}

#[derive(Template, Debug)]
#[template(path = "repository/federated/view.html")]
pub struct FederatedRepositoryTemplate {
    repository: String,
    owner: String,
    stats: RepositoryStats,
    user: Option<String>,
}

#[derive(Template, Debug)]
#[template(path = "repository/tree/tree.html")]
pub struct GitTreeTemplate {
    repository: String,
    owner: String,
    stats: RepositoryStats,
    tree: TreeDto,
    readme: Option<String>,
    branches: Vec<BranchDto>,
    current_branch: Option<String>,
    user: Option<String>,
}

/// Returns a tree with for a given owner, repository and a branch
pub async fn tree(
    user: Option<Oauth2User>,
    Path((owner, repository, branch)): Path<(String, String, String)>,
    Path(path): Path<Vec<String>>,
    Extension(db): Extension<PgPool>,
) -> AppResult<HtmlTemplate<GitTreeTemplate>> {
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
) -> AppResult<HtmlTemplate<GitTreeTemplate>> {
    let connected_username = get_connected_user_username(&db, user).await;
    imp::get_tree_root(&owner, &repository, branch, connected_username, &db).await
}

/// Returns the root of a tree with for a given owner and repository
/// using the default branch
pub async fn root(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> AppResult<DynHtmlTemplate<Box<dyn DynTemplate>>> {
    let connected_username = get_connected_user_username(&db, user).await;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;

    if repo.is_local {
        let branch = repo
            .get_default_branch(&db)
            .await
            .ok_or_else(|| anyhow!("No default branch"))?;

        let template =
            imp::get_tree_root(&owner, &repository, branch.name, connected_username, &db).await?;
        Ok(DynHtmlTemplate(Box::new(template.inner())))
    } else {
        Ok(DynHtmlTemplate(Box::new(FederatedRepositoryTemplate {
            repository: repository.to_string(),
            owner: owner.to_string(),
            stats,
            user: connected_username,
        })))
    }
}

mod imp {
    use super::GitTreeTemplate;
    use crate::error::AppResult;
    use crate::view::repository::tree::TreeDto;
    use crate::view::HtmlTemplate;

    use crate::domain::repository::stats::RepositoryStats;
    use crate::view::repository::get_repository_branches;

    use gill_git::traversal::TreeEntry;
    use gill_git::GitRepository;
    use sqlx::PgPool;

    pub(crate) async fn get_tree_root(
        owner: &str,
        repository: &str,
        current_branch: String,
        connected_username: Option<String>,
        db: &PgPool,
    ) -> AppResult<HtmlTemplate<GitTreeTemplate>> {
        let repo = GitRepository::open(owner, repository)?;
        let tree = repo.get_tree_for_path(Some(&current_branch), None)?;
        let readme = get_readme(&tree, &repo, owner, repository);
        let tree = TreeDto::from(tree);
        let branches = get_repository_branches(owner, repository, &current_branch, db).await?;
        let stats = RepositoryStats::get(owner, repository, db).await?;

        let template = GitTreeTemplate {
            repository: repository.to_string(),
            owner: owner.to_string(),
            stats,
            tree,
            readme,
            branches,
            current_branch: Some(current_branch),
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
    ) -> AppResult<HtmlTemplate<GitTreeTemplate>> {
        let repo = GitRepository::open(&owner, &repository)?;
        let tree = repo.get_tree_for_path(Some(&current_branch), tree_path)?;
        let readme = get_readme(&tree, &repo, &owner, &repository);
        let tree = TreeDto::from(tree);
        let branches = get_repository_branches(&owner, &repository, &current_branch, db).await?;
        let stats = RepositoryStats::get(&owner, &repository, db).await?;

        let template = GitTreeTemplate {
            repository,
            owner,
            stats,
            tree,
            readme,
            branches,
            current_branch: Some(current_branch),
            user: connected_username,
        };

        Ok(HtmlTemplate(template))
    }

    pub fn get_readme(
        tree: &TreeEntry,
        repo: &GitRepository,
        owner: &str,
        repository_name: &str,
    ) -> Option<String> {
        tree.blobs
            .iter()
            .find(|blob| &blob.filename() == "README.md")
            .and_then(|blob| repo.blob_str(blob).ok())
            .map(|readme| gill_markdown::render(&readme, owner, repository_name))
    }
}
