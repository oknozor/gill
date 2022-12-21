use crate::state::AppState;
use anyhow::Result;
use axum::routing::{get, post};
use axum::Router;
use gill_db::user::User;
use sqlx::PgPool;
use std::fmt;
use std::fmt::Formatter;

pub mod activity;
pub mod blob;
pub mod commits;
pub mod diff;
pub mod issues;
pub mod pulls;
pub mod tree;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:owner/:repository", get(tree::root))
        .route("/:owner/:repository/issues", get(issues::issues))
        .route("/:owner/:repository/pulls", get(pulls::pulls))
        .route("/:owner/:repository/tree/:branch", get(tree::tree_root))
        .route("/:owner/:repository/tree/:branch/*tree", get(tree::tree))
        .route("/:owner/:repository/blob/:branch/*blob", get(blob::blob))
        .route(
            "/:owner/:repository/commits/:branch/",
            get(commits::history),
        )
        .route("/:owner/:repository/commits/:branch", get(commits::history))
        .route("/:owner/:repository/diff", get(diff::diff))
        .route("/:owner/:repository/star", post(activity::star))
        .route("/:owner/:repository/watch", post(activity::watch))
}

#[derive(Debug)]
pub struct BranchDto {
    name: String,
    is_default: bool,
    is_current: bool,
}

async fn get_repository_branches(
    owner: &str,
    repository: &str,
    current_branch: &str,
    db: &PgPool,
) -> Result<Vec<BranchDto>> {
    let user = User::by_user_name(owner, db).await.unwrap();
    let repository = user.get_local_repository_by_name(repository, db).await?;
    let branches = repository.list_branches(20, 0, db).await?;
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

    Ok(branches)
}

impl fmt::Display for BranchDto {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self:?}")
    }
}
