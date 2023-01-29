use crate::GitRepository;
use git_repository::Commit;

impl GitRepository {
    pub fn commit_by_sha(&self, sha: &str) -> anyhow::Result<OwnedCommit> {
        self.find_commit(sha)
    }

    pub fn history(&self, branch: &str) -> anyhow::Result<Vec<OwnedCommit>> {
        self.list_commits(branch)
    }

    pub fn history_between(&self, base: &str, compare: &str) -> anyhow::Result<Vec<OwnedCommit>> {
        self.list_commits_between_ref(base, compare)
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

impl TryFrom<&Commit<'_>> for OwnedCommit {
    type Error = anyhow::Error;

    fn try_from(commit: &Commit<'_>) -> Result<Self, Self::Error> {
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
    use git_repository::ObjectId;

    impl GitRepository {
        pub fn find_commit(&self, sha: &str) -> Result<OwnedCommit> {
            let object_id = ObjectId::from_hex(sha.as_bytes())?;
            let commit = self.inner.find_object(object_id)?.try_into_commit()?;
            OwnedCommit::try_from(&commit)
        }

        pub fn list_commits(&self, refs: &str) -> Result<Vec<OwnedCommit>> {
            let refs = format!("refs/heads/{refs}");
            let tree = self.inner.find_reference(&refs)?;
            let target_ref = tree.target();
            let commit = self.inner.find_object(target_ref.id())?.try_into_commit()?;
            let mut commits = vec![];
            for commit in commit.ancestors().all()? {
                let commit = commit?.object()?.try_into_commit()?;
                let commit = OwnedCommit::try_from(&commit)?;
                commits.push(commit);
            }

            Ok(commits)
        }

        pub fn list_commits_between_ref(
            &self,
            base: &str,
            compare: &str,
        ) -> Result<Vec<OwnedCommit>> {
            let base = format!("refs/heads/{base}");
            let compare = format!("refs/heads/{compare}");
            let base_tree = self.inner.find_reference(&base)?;
            let compare_tree = self.inner.find_reference(&compare)?;
            let target_ref_base = base_tree.target();
            let target_ref_compare = compare_tree.target();
            let base_head = self
                .inner
                .find_object(target_ref_base.id())?
                .try_into_commit()?;
            let compare_head = self
                .inner
                .find_object(target_ref_compare.id())?
                .try_into_commit()?;
            let mut commits = vec![];
            for commit in compare_head.ancestors().all()? {
                let id = commit?;
                if id == base_head.id {
                    break;
                }
                let commit = id.object()?.try_into_commit()?;
                let commit = OwnedCommit::try_from(&commit)?;
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
        let commits = repo.list_commits("master")?;

        // Assert
        assert_that!(commits).has_length(3);
        Ok(())
    }

    #[sealed_test]
    fn list_commit_between_branches() -> anyhow::Result<()> {
        // Arrange
        run_cmd!(
            git init;
            git commit --allow-empty -m "one";
            git commit --allow-empty -m "two";
            git commit --allow-empty -m "three";
            git checkout -b other;
            git commit --allow-empty -m "other_one";
            git commit --allow-empty -m "other_two";
        )?;

        let repo = GitRepository {
            inner: git_repository::open(".")?,
        };

        // Act
        let commits = repo.list_commits_between_ref("master", "other")?;

        // Assert
        assert_that!(commits).has_length(2);
        assert_that!(commits[1].summary).is_equal_to(&"other_one".to_string());
        assert_that!(commits[0].summary).is_equal_to(&"other_two".to_string());
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
