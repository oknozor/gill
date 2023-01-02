use crate::error::AppError;
use crate::view::repository::tree_and_blob_from_query;
use axum::extract::Path;
use axum::Extension;
use gill_db::repository::Repository;
use gill_git::traversal::BlobMime;
use gill_git::GitRepository;
use sqlx::PgPool;

pub async fn image(
    Path((owner, repository)): Path<(String, String)>,
    Path(path): Path<Vec<String>>,
    Extension(db): Extension<PgPool>,
) -> Result<Vec<u8>, AppError> {
    let path = path.last().unwrap();
    let (tree, blob_name) = tree_and_blob_from_query(path);
    let repo = GitRepository::open(&owner, &repository)?;
    let repo_entity = Repository::by_namespace(&owner, &repository, &db).await?;
    let branch = repo_entity
        .get_default_branch(&db)
        .await
        .ok_or(AppError::NotFound)?;
    let tree = repo.get_tree_for_path(Some(&branch.name), tree)?;
    let blob = tree
        .blobs
        .iter()
        .find(|blob| blob.filename() == blob_name)
        .unwrap();
    let blob = match repo.blob_mime(blob) {
        BlobMime::Image => repo.blob_bytes(blob).ok(),
        _ => None,
    };

    blob.ok_or(AppError::NotFound)
}
