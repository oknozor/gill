use crate::GitRepository;
use git_repository::Commit;
use std::path::PathBuf;

impl GitRepository {
    pub fn by_sha(&self, sha: &str) -> anyhow::Result<OwnedCommit> {
        self.find_commit(sha)
    }

    pub fn history(&self) -> anyhow::Result<Vec<OwnedCommit>> {
        self.list_commits()
    }
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
    use crate::GitRepository;
    use anyhow::Result;
    use git_repository::{ObjectId, Repository};

    impl GitRepository {
        pub fn find_commit(&self, sha: &str) -> Result<OwnedCommit> {
            let object_id = ObjectId::from_hex(sha.as_bytes())?;
            let commit = self.inner.find_object(object_id)?.try_into_commit()?;
            OwnedCommit::try_from(commit)
        }

        pub fn list_commits(&self) -> Result<Vec<OwnedCommit>> {
            let head = self.inner.head_commit()?;
            let mut commits = vec![];
            for commit in head.ancestors().all()? {
                let commit = commit?.object()?.try_into_commit()?;
                let commit = OwnedCommit::try_from(commit)?;
                commits.push(commit);
            }

            Ok(commits)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::commits::OwnedCommit;
    use crate::GitRepository;
    use anyhow::anyhow;
    use cmd_lib::{run_cmd, run_fun};
    use sealed_test::prelude::*;
    use speculoos::prelude::*;

    // Helper function to create a commit and get its sha1
    fn git_commit(message: &str) -> anyhow::Result<String> {
        run_fun!(
            git commit --allow-empty -q -m $message;
            git log --format=%H -n 1;
        )
        .map_err(|e| anyhow!(e))
    }

    #[sealed_test]
    fn list_repository_commits() -> anyhow::Result<()> {
        // Arrange
        run_cmd!(
            git init;
            git commit --allow-empty -m "one";
            git commit --allow-empty -m "two";
            git commit --allow-empty -m "three";
        )?;

        let repo = GitRepository {
            inner: git_repository::open(".")?,
        };

        // Act
        let commits = repo.list_commits()?;

        // Assert
        assert_that!(commits).has_length(3);
        Ok(())
    }

    #[sealed_test]
    fn find_commit_ok() -> anyhow::Result<()> {
        // Arrange
        run_cmd!(git init;)?;
        git_commit("one")?;
        let sha1 = git_commit("two")?;
        git_commit("three")?;

        let repo = GitRepository {
            inner: git_repository::open(".")?,
        };

        // Act
        let commit = repo.find_commit(&sha1);

        // Assert
        assert_that!(commit)
            .is_ok()
            .map(|commit: &OwnedCommit| &commit.summary)
            .is_equal_to(&"two".to_string());

        Ok(())
    }
}
