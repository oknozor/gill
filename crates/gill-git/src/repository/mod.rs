use git_repository::{Repository, Tree};
use std::path::PathBuf;
use std::str::FromStr;

pub mod commits;
pub mod diff;
pub mod init;
pub mod traversal;

const REPO_DIR: &str = "/home/git";

// TODO: add namespace params.
// We need to wrap 'git_repository::open' so we can survive it's lifetime :)
pub fn open(owner: &str, repository: &str) -> anyhow::Result<Repository> {
    let path = PathBuf::from_str(owner)?.join(format!("{repository}.git"));

    Ok(git_repository::open(path)?)
}

pub fn ref_to_tree<'repo>(
    reference: Option<&str>,
    repo: &'repo git_repository::Repository,
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
