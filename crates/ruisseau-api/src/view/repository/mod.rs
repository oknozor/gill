use crate::oauth::AppState;
use axum::routing::get;
use axum::Router;
use std::fmt;
use std::fmt::Formatter;

pub mod blob;
pub mod tree;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:owner/:repository", get(tree::root))
        .route("/:owner/:repository/tree/:branch", get(tree::tree_root))
        .route("/:owner/:repository/tree/:branch/*tree", get(tree::tree))
        .route("/:owner/:repository/blob/:branch/*blob", get(blob::blob))
}

#[derive(Debug)]
pub struct BranchDto {
    name: String,
    is_default: bool,
    is_current: bool,
}

impl fmt::Display for BranchDto {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}
