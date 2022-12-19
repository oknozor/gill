use git_repository::{Repository, Tree};
use std::path::PathBuf;


pub mod commits;
pub mod diff;
pub mod init;
pub mod traversal;

const REPO_DIR: &str = "/home/git";

#[derive(Debug)]
pub struct GitRepository {
    inner: Repository,
}

impl GitRepository {
    pub fn open(owner: &str, name: &str) -> anyhow::Result<Self> {
        let path = PathBuf::from(REPO_DIR)
            .join(owner)
            .join(format!("{name}.git"));
        Ok(Self {
            inner: git_repository::open(path)?,
        })
    }
}

pub fn ref_to_tree<'repo>(
    reference: Option<&str>,
    repo: &'repo Repository,
) -> anyhow::Result<Tree<'repo>> {
    Ok(match reference {
        Some(reference) => repo
            .find_reference(reference)?
            .peel_to_id_in_place()?
            .object()?
            .try_into_commit()?
            .tree()?,
        None => repo.head()?.peel_to_commit_in_place()?.tree()?,
    })
}
