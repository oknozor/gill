use crate::GitRepository;
use cmd_lib::{init_builtin_logger, run_cmd};

impl GitRepository {
    pub fn merge(&self, base: &str, compare: &str) -> anyhow::Result<()> {
        let non_bare = self.get_or_create_non_bare()?;
        let path = non_bare.path();

        // TODO: Merge is not yet implemented in git-oxide
        //  this should be replaced when ready
        run_cmd!(
            cd $path;
            git checkout $base;
            git merge --no-ff origin/$compare;
            git push -u origin $base;
        )?;

        Ok(())
    }

    pub fn rebase(&self, base: &str, compare: &str) -> anyhow::Result<()> {
        let non_bare = self.get_or_create_non_bare()?;
        let path = non_bare.path();

        // TODO: Rebase is not yet implemented in git-oxide
        //  this should be replaced when ready
        run_cmd!(
            cd $path;
            git checkout $base;
            git rebase origin/$compare;
            git push -u origin $base;
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::GitRepository;
    use cmd_lib::{init_builtin_logger, run_cmd};
    use sealed_test::prelude::*;
    use speculoos::prelude::*;
    use std::fs;

    #[sealed_test]
    fn should_merge_branches() -> anyhow::Result<()> {
        init_builtin_logger();
        run_cmd!(
            git init --bare repo;
            git init work_repo;
            cd work_repo;
            git remote add origin ../repo;
            git commit --allow-empty -m "First commit";
            git push -u origin master;
            git checkout -b other;
        )?;

        fs::write("work_repo/file", "changes")?;

        run_cmd!(
            cd work_repo;
            git add .;
            git commit -m "commit a";
            git push -u origin other;
        )?;

        let repo = GitRepository {
            inner: git_repository::open("repo")?,
        };

        let merge = repo.merge("master", "other");

        assert_that!(repo.inner.is_bare()).is_true();
        println!("{:?}", repo.list_commits());
        assert_that!(repo.list_commits()).is_ok().has_length(3);

        Ok(())
    }

    #[sealed_test]
    fn should_rebase_branches() -> anyhow::Result<()> {
        init_builtin_logger();
        run_cmd!(
            git init --bare repo;
            git init work_repo;
            cd work_repo;
            git remote add origin ../repo;
            git commit --allow-empty -m "First commit";
            git push -u origin master;
            git checkout -b other;
        )?;

        fs::write("work_repo/file", "changes")?;

        run_cmd!(
            cd work_repo;
            git add .;
            git commit -m "commit a";
            git push -u origin other;
        )?;

        let repo = GitRepository {
            inner: git_repository::open("repo")?,
        };

        let merge = repo.rebase("master", "other");

        assert_that!(repo.inner.is_bare()).is_true();
        println!("{:?}", repo.list_commits());
        assert_that!(repo.list_commits()).is_ok().has_length(2);

        Ok(())
    }
}
