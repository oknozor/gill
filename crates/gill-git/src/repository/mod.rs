use git_repository::Tree;

pub mod commits;
pub mod diff;
pub mod init;
pub mod traversal;

const REPO_DIR: &str = "/home/git";

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
