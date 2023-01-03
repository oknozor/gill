use crate::GitRepository;

use std::process::Command;

impl GitRepository {
    pub fn merge(
        &self,
        base: &str,
        compare: &str,
        username: &str,
        email: &str,
    ) -> anyhow::Result<()> {
        self.get_or_create_non_bare(username, email)?;
        let path = self.non_bare_path();

        // TODO: Merge is not yet implemented in git-oxide
        //  this should be replaced when ready
        Command::new("git")
            .current_dir(&path)
            .args(["checkout", base])
            .output()
            .expect("Failed to checkout branch");

        Command::new("git")
            .current_dir(&path)
            .args(["fetch"])
            .output()
            .expect("Failed to fetch branch");

        Command::new("git")
            .current_dir(&path)
            .args(["reset", "--hard", &format!("origin/{base}")])
            .output()
            .expect("Failed to reset branch to bare state");

        Command::new("git")
            .current_dir(&path)
            .args([
                "merge",
                "--no-ff",
                "--no-edit",
                &format!("origin/{compare}"),
            ])
            .output()
            .expect("Failed to merge branch");

        Command::new("git")
            .current_dir(&path)
            .args(["push", "-u", "origin", base])
            .output()
            .expect("Failed to sync bare repo");

        Ok(())
    }

    pub fn rebase(
        &self,
        base: &str,
        compare: &str,
        username: &str,
        email: &str,
    ) -> anyhow::Result<()> {
        self.get_or_create_non_bare(username, email)?;
        let path = self.non_bare_path();

        // TODO: Rebase is not yet implemented in git-oxide
        //  this should be replaced when ready
        Command::new("git")
            .current_dir(&path)
            .args(["checkout", base])
            .output()
            .expect("Failed to checkout branch");

        Command::new("git")
            .current_dir(&path)
            .args(["fetch"])
            .output()
            .expect("Failed to fetch branch");

        Command::new("git")
            .current_dir(&path)
            .args(["reset", "--hard", &format!("origin/{base}")])
            .output()
            .expect("Failed to reset branch to bare state");

        Command::new("git")
            .current_dir(&path)
            .args(["rebase", &format!("origin/{compare}")])
            .output()
            .expect("Failed to rebase branch");

        Command::new("git")
            .current_dir(&path)
            .args(["push", "-u", "origin", base])
            .output()
            .expect("Failed to sync bare repo");

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

        let _merge = repo.merge("master", "other", "gill", "gill@test.org");

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

        let _merge = repo.rebase("master", "other", "gill", "gill@test.org");

        assert_that!(repo.inner.is_bare()).is_true();
        assert_that!(repo.list_commits()).is_ok().has_length(2);

        Ok(())
    }
}
