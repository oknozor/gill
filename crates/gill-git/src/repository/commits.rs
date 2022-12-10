use std::path::PathBuf;
use git_repository::Commit;
use git_repository::object::Kind;
use git_repository::traverse::commit::Sorting;
use crate::repository::commits::imp::list_commits;
use crate::repository::REPO_DIR;

#[derive(Debug)]
pub struct OwnedCommit {
    id: String,
}

pub fn history<'a>(namespace: &'a str, name: &'a str) -> anyhow::Result<Vec<OwnedCommit>> {
    let path = PathBuf::from(REPO_DIR).join(namespace).join(name);
    let repo = git_repository::open(path)?;
    list_commits(&repo)
}

mod imp {
    use git_repository::{Commit, Repository};
    use crate::repository::commits::OwnedCommit;

    pub fn list_commits(repo: &Repository) -> anyhow::Result<Vec<OwnedCommit>> {
        let head = repo.head_commit()?;
        let mut commits = vec![];
        for commit in head.ancestors().all()? {
            let commit = commit?
                .object()?
                .try_into_commit()?;
            let commit = OwnedCommit {
                id: commit.id.to_string(),
            };

            commits.push(commit);
        }

        Ok(commits)
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::path::PathBuf;
    use git_repository::Repository;
    use crate::repository::commits::imp::list_commits;

    #[test]
    fn list_repository_commits() -> anyhow::Result<()> {
        let repo = git_repository::open("/home/okno/Code/gill")?;
        let commits = list_commits(&repo)?;
        println!("{}", commits.len());
        Ok(())
    }
}
