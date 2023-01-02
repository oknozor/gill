use crate::GitRepository;
use cmd_lib::run_cmd;
use git_repository::clone::PrepareFetch;
use git_repository::progress::Discard;
use git_repository::{create, interrupt, open, worktree};

impl GitRepository {
    pub(crate) fn get_or_create_non_bare(
        &self,
        username: &str,
        email: &str,
    ) -> anyhow::Result<GitRepository> {
        // If the non bare copy already exist, returns it early
        if self.has_non_bare_clone() {
            let mut path = self.path();
            path.pop();
            let path = path.join("non-bare-copy");
            let non_bare = GitRepository {
                inner: git_repository::open(path)?,
            };

            return Ok(non_bare);
        };

        // else we create the non bare repo
        let repository_path = self.inner.path();
        let mut dest = self.inner.path().to_path_buf();
        dest.pop();
        let dest = dest.join("non-bare-copy");
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

        Ok(GitRepository {
            inner: git_repository::open(dest_copy)?,
        })
    }

    pub(crate) fn has_non_bare_clone(&self) -> bool {
        let mut path = self.inner.path().to_path_buf();
        path.pop();
        path.join("non-bare-copy").exists()
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
        let non_bare = repository.get_or_create_non_bare("gill", "gill@test.org")?;

        // Assert
        let commits = non_bare.list_commits()?;
        assert_that!(non_bare.path()).is_equal_to(&PathBuf::from("non-bare-copy"));
        assert_that!(repository.has_non_bare_clone()).is_true();
        assert_that!(commits).has_length(1);

        Ok(())
    }
}
