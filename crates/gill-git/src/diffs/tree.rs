use crate::diffs::Diff;
use crate::{ref_to_tree, GitRepository};

impl GitRepository {
    pub fn diff(&self, branch: &str, other: &str) -> anyhow::Result<Vec<Diff>> {
        let repository = &self.inner;
        let tree = ref_to_tree(Some(&format!("heads/{branch}")), repository)?;
        let other = ref_to_tree(Some(&format!("heads/{other}")), repository)?;
        self.diff_tree_to_tree(tree, other)
    }
}

#[cfg(test)]
mod test {
    use crate::GitRepository;
    use anyhow::Result;
    use cmd_lib::run_cmd;
    use sealed_test::prelude::*;
    use speculoos::prelude::*;
    use std::fs;

    #[sealed_test]
    fn should_get_diff() -> Result<()> {
        // Arrange
        run_cmd!(git init;)?;
        fs::write("file", "changes")?;
        run_cmd!(
            git add .;
            git commit -m "first commit";
            git checkout -b other;
        )?;
        fs::write("file2", "changes")?;
        run_cmd!(
            git add .;
            git commit -m "second commit";
        )?;

        let repo = GitRepository {
            inner: git_repository::open(".")?,
        };

        // Act
        let diffs = repo.diff("master", "other");

        // Assert
        assert_that!(diffs).is_ok().has_length(1);

        Ok(())
    }
}
