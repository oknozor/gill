use crate::oauth::AppState;
use axum::routing::get;
use axum::Router;

pub mod blob;
pub mod tree;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:owner/:repository", get(tree::tree_root))
        .route("/:owner/:repository/tree/:branch", get(tree::tree_root))
        .route("/:owner/:repository/tree/:branch/*tree", get(tree::tree))
        .route("/:owner/:repository/blob/:branch/*blob", get(blob::blob))
}
