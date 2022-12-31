use crate::REPO_DIR;
use git_repository::Commit;
use std::path::PathBuf;

pub fn by_sha<'a>(namespace: &'a str, name: &'a str, sha: &'a str) -> anyhow::Result<OwnedCommit> {
    let path = PathBuf::from(REPO_DIR).join(namespace).join(name);
    let repo = git_repository::open(path)?;
    imp::find_commit(&repo, sha)
}

pub fn history<'a>(namespace: &'a str, name: &'a str) -> anyhow::Result<Vec<OwnedCommit>> {
    let path = PathBuf::from(REPO_DIR)
        .join(namespace)
        .join(format!("{name}.git"));
    let repo = git_repository::open(path)?;
    imp::list_commits(&repo)
}

#[derive(Debug, Clone)]
pub struct OwnedCommit {
    pub id: String,
    pub summary: String,
    pub body: Option<String>,
    pub author: String,
    pub email: String,
    pub created_at: u32,
    pub authored_at: u32,
}

impl TryFrom<Commit<'_>> for OwnedCommit {
    type Error = anyhow::Error;

    fn try_from(commit: Commit<'_>) -> Result<Self, Self::Error> {
        let message_ref = commit.message()?;
        let id = commit.id.to_string();
        let summary = message_ref.summary().to_string();
        let body = message_ref.body.map(ToString::to_string);
        let created_at = commit.time()?.seconds();
        let signature_ref = commit.author()?;
        let authored_at = signature_ref.time.seconds();
        let author = signature_ref.name.to_string();
        let email = signature_ref.email.to_string();

        Ok(OwnedCommit {
            id,
            summary,
            body,
            author,
            email,
            created_at,
            authored_at,
        })
    }
}

mod imp {
    use crate::commits::OwnedCommit;
    use anyhow::Result;
    use git_repository::{ObjectId, Repository};

    pub fn find_commit(repo: &Repository, sha: &str) -> Result<OwnedCommit> {
        let object_id = ObjectId::from_hex(sha.as_bytes())?;
        let commit = repo.find_object(object_id)?.try_into_commit()?;
        OwnedCommit::try_from(commit)
    }

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
    use crate::commits::imp::{find_commit, list_commits};

    #[test]
    // TODO: Make some actual assertion here
    //  need sealed test to setup a fake repository
    fn list_repository_commits() -> anyhow::Result<()> {
        let repo = git_repository::open("/home/okno/Code/gill")?;
        let commits = list_commits(&repo)?;
        for x in commits {
            println!("{x:#?}");
        }
        Ok(())
    }

    #[test]
    // TODO: Make some actual assertion here
    //  need sealed test to setup a fake repository
    fn find_commit_ok() -> anyhow::Result<()> {
        let repo = git_repository::open("/home/okno/Code/gill")?;
        let commit = find_commit(&repo, "4dfecc6603e9332279a83f045b7afbd4c6f5816e");
        assert!(commit.is_ok());
        Ok(())
    }
}
