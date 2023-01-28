use git_repository::{Commit, Id, Repository, Tree};
use std::path::PathBuf;

pub mod clone;
pub mod commits;
pub mod diffs;
pub mod init;
pub mod merge;
pub mod ssh;
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

    pub(crate) fn path(&self) -> PathBuf {
        self.inner.path().to_path_buf()
    }

    pub(crate) fn non_bare_path(&self) -> PathBuf {
        let mut path = self.inner.path().to_path_buf();
        if !self.inner.is_bare() {
            return self.path();
        }

        let filename = path
            .file_name()
            .expect("filename")
            .to_string_lossy()
            .to_string();

        path.pop();

        let path = path.join(format!("non-bare-copy-{filename}"));
        path
    }
}

pub(crate) fn ref_to_tree<'repo>(
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

pub(crate) fn id_to_commit<'a>(id: &'a Id) -> anyhow::Result<Commit<'a>> {
    let object = id.try_object()?;
    let object = object.expect("empty");
    let commit = object.try_into_commit()?;
    Ok(commit)
}

#[cfg(test)]
mod test {
    use crate::GitRepository;
    use cmd_lib::run_cmd;
    use sealed_test::prelude::*;
    use speculoos::prelude::*;
    use std::path::PathBuf;

    #[sealed_test]
    fn should_get_repository_path() -> anyhow::Result<()> {
        run_cmd!(git init repo;)?;

        let repository = GitRepository {
            inner: git_repository::open("repo")?,
        };

        assert_that!(repository.path()).is_equal_to(&PathBuf::from("repo/.git"));
        Ok(())
    }

    #[sealed_test]
    fn should_get_bare_repository_path() -> anyhow::Result<()> {
        run_cmd!(git init --bare repo;)?;

        let repository = GitRepository {
            inner: git_repository::open("repo")?,
        };

        assert_that!(repository.path()).is_equal_to(&PathBuf::from("repo"));
        Ok(())
    }

    #[sealed_test]
    fn should_append_ssh_key() -> anyhow::Result<()> {
        run_cmd!(git init --bare repo;)?;

        let repository = GitRepository {
            inner: git_repository::open("repo")?,
        };

        assert_that!(repository.path()).is_equal_to(&PathBuf::from("repo"));
        Ok(())
    }
}
