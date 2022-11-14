use std::path::PathBuf;
use git_repository::init::Error;
use git_repository::Repository;

mod init;

const REPO_BASE_PATH: &str = "/git-server/repos/";

pub fn init_bare(name: &str) -> Result<Repository, Error> {
    init::bare(PathBuf::from(REPO_BASE_PATH), name)
}

