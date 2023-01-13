use crate::GitRepository;
use cmd_lib::run_cmd;
use git_repository::clone::PrepareFetch;
use git_repository::progress::Discard;
use git_repository::{create, interrupt, open, worktree};

impl GitRepository {
    pub(crate) fn get_or_create_non_bare(&self, username: &str, email: &str) -> anyhow::Result<()> {
        // If the non bare copy already exist, returns it early
        let non_bare = self.non_bare_path();
        if non_bare.exists() {
            return Ok(());
        };

        // else we create the non bare repo
        let repository_path = self.inner.path();
        let dest = self.non_bare_path();
        let dest_copy = dest.clone();
        let mut prepare = PrepareFetch::new(
            repository_path,
            dest,
            create::Kind::WithWorktree,
            create::Options::default(),
            {
                let mut opts = open::Options::default();
                opts.permissions.config.git_binary = true;
                opts
            },
        )?;

        let (mut checkout, _) = prepare.fetch_then_checkout(Discard, &interrupt::IS_INTERRUPTED)?;
        let (_, outcome) = checkout.main_worktree(Discard, &interrupt::IS_INTERRUPTED)?;
        let worktree::index::checkout::Outcome {
            collisions, errors, ..
        } = outcome;

        if !(collisions.is_empty() && errors.is_empty()) {
            if !errors.is_empty() {
                for record in errors {
                    eprintln!("{}: {}", record.path, record.error);
                }
            }
            if !collisions.is_empty() {
                for col in collisions {
                    eprintln!("{}: collision ({:?})", col.path, col.error_kind);
                }
            }
        }

        run_cmd!(
            cd $dest_copy;
            git config user.email $email;
            git config user.name $username;
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::GitRepository;
    use cmd_lib::run_cmd;
    use sealed_test::prelude::*;
    use speculoos::prelude::*;
    use std::fs;
    use std::path::PathBuf;

    #[sealed_test]
    fn non_bare_copy() -> anyhow::Result<()> {
        // Arrange
        run_cmd!(
            git init --bare repository;
            git init base_repository;
            cd base_repository;
        )?;

        fs::write("base_repository/files", "changes")?;

        run_cmd!(
            cd base_repository;
            git add .;
            git commit -m "first commit";
            git remote add bare ../repository;
            git push -u bare master;
            cd ..;
        )?;

        let repository = GitRepository {
            inner: git_repository::open("repository")?,
        };

        // Act
        repository.get_or_create_non_bare("gill", "gill@test.org")?;

        // Assert
        let non_bare = GitRepository {
            inner: git_repository::open("non-bare-copy-repository")?,
        };
        let commits = non_bare.list_commits("master")?;
        assert_that!(non_bare.path()).is_equal_to(&PathBuf::from("non-bare-copy-repository"));
        assert_that!(repository.non_bare_path()).exists();
        assert_that!(commits).has_length(1);

        Ok(())
    }
}
