use git_repository::{Commit, Id, Repository, Tree};
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

pub mod clone;
pub mod commits;
pub mod diff;
pub mod init;
pub mod merge;
pub mod traversal;

const REPO_DIR: &str = "/home/git";

pub fn append_ssh_key(ssh_key: &str, user_id: i32) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true) // This is needed to append to file
        .open("/home/git/.ssh/authorized_keys")?;

    write!(
        file,
        r#"command="./bin/gill-git-server {user_id}" {ssh_key}"#
    )
}

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
        if self.inner.is_bare() {
            self.inner.path().to_path_buf()
        } else {
            let mut path = self.inner.path().to_path_buf();
            path.pop();
            path
        }
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

        assert_that!(repository.path()).is_equal_to(&PathBuf::from("repo"));
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
}
