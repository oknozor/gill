use crate::diffs::Diff;
use crate::GitRepository;
use git_repository::{Id, ObjectId};

impl GitRepository {
    pub fn commit_diff(&self, sha: &str) -> anyhow::Result<Vec<Diff>> {
        let object_id = ObjectId::from_hex(sha.as_bytes())?;
        let commit = self.inner.find_object(object_id)?.try_into_commit()?;
        let parents: Vec<Id> = commit.parent_ids().collect();
        let parent = parents.first();
        let parent_tree = match parent {
            None => self.inner.empty_tree(),
            Some(parent_id) => {
                let parent = parent_id.object()?;
                parent.peel_to_tree()?
            }
        };

        self.diff_tree_to_tree(parent_tree, commit.tree()?)
    }
}

#[cfg(test)]
mod test {
    use crate::GitRepository;
    use anyhow::{anyhow, Result};
    use cmd_lib::{run_cmd, run_fun};
    use sealed_test::prelude::*;
    use speculoos::prelude::*;
    use std::fs;

    // Helper function to create a commit and get its sha1
    fn git_commit(message: &str) -> anyhow::Result<String> {
        run_fun!(
            git commit --allow-empty -q -m $message;
            git log --format=%H -n 1;
        )
        .map_err(|e| anyhow!(e))
    }

    #[sealed_test]
    fn should_get_diff_when_commit_has_parent() -> Result<()> {
        // Arrange
        run_cmd!(git init;)?;
        fs::write("file", "changes")?;
        run_cmd!(git add .;)?;
        let _ = git_commit("first commit")?;
        fs::write("file2", "changes")?;
        run_cmd!(git add .;)?;
        let commit_two = git_commit("second commit")?;

        let repo = GitRepository {
            inner: git_repository::open(".")?,
        };

        // Act
        let diffs = repo.commit_diff(&commit_two);

        // Assert
        assert_that!(diffs).is_ok().has_length(1);

        Ok(())
    }

    #[sealed_test]
    fn should_get_diff_when_without_commit_parent() -> Result<()> {
        // Arrange
        run_cmd!(git init;)?;
        fs::write("file", "changes")?;
        run_cmd!(git add .;)?;
        let commit = git_commit("first commit")?;

        let repo = GitRepository {
            inner: git_repository::open(".")?,
        };

        // Act
        let diffs = repo.commit_diff(&commit);

        // Assert
        println!("{diffs:?}");
        assert_that!(diffs).is_ok().has_length(1);

        Ok(())
    }
}
