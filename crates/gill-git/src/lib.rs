use git_repository::{Repository, Tree};
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
