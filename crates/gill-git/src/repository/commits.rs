use crate::repository::REPO_DIR;
use git_repository::Commit;
use std::path::PathBuf;

pub fn history<'a>(namespace: &'a str, name: &'a str) -> anyhow::Result<Vec<OwnedCommit>> {
    let path = PathBuf::from(REPO_DIR).join(namespace).join(name);
    let repo = git_repository::open(path)?;
    imp::list_commits(&repo)
}

#[derive(Debug)]
pub struct OwnedCommit {
    pub id: String,
    pub summary: String,
    pub body: Option<String>,
    pub author: String,
    pub email: String,
    pub time: u32,
}

impl TryFrom<Commit<'_>> for OwnedCommit {
    type Error = anyhow::Error;

    fn try_from(commit: Commit<'_>) -> Result<Self, Self::Error> {
        let message_ref = commit.message()?;
        let id = commit.id.to_string();
        let summary = message_ref.summary().to_string();
        let body = message_ref.body.map(ToString::to_string);
        let time = commit.time()?.seconds();
        let signature_ref = commit.author()?;
        let author = signature_ref.name.to_string();
        let email = signature_ref.email.to_string();

        Ok(OwnedCommit {
            id,
            summary,
            body,
            author,
            email,
            time,
        })
    }
}

mod imp {
    use crate::repository::commits::OwnedCommit;
    use anyhow::Result;
    use git_repository::Repository;

    pub fn list_commits(repo: &Repository) -> Result<Vec<OwnedCommit>> {
        let head = repo.head_commit()?;
        let mut commits = vec![];
        for commit in head.ancestors().all()? {
            let commit = commit?.object()?.try_into_commit()?;
            let commit = OwnedCommit::try_from(commit)?;
            commits.push(commit);
        }

        Ok(commits)
    }
}

#[cfg(test)]
mod test {
    use crate::repository::commits::imp::list_commits;

    #[test]
    // TODO: Make some actual assertion here
    // need sealed test to setup a fake repository
    fn list_repository_commits() -> anyhow::Result<()> {
        let repo = git_repository::open("/home/okno/Code/gill")?;
        let commits = list_commits(&repo)?;
        for x in commits {
            println!("{:#?}", x);
        }
        Ok(())
    }
}
