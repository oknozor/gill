use crate::{GitRepository, REPO_DIR};

use std::fs;
use std::path::PathBuf;

pub fn init_bare(namespace: &str, name: &str) -> anyhow::Result<GitRepository> {
    let path = PathBuf::from(REPO_DIR).join(namespace);

    if !path.exists() {
        fs::create_dir(&path).expect("Failed to create dir");
    }

    GitRepository::init_bare(path, name)
}

impl GitRepository {
    pub fn list_branches(&self) -> anyhow::Result<Vec<String>> {
        let refs = self.inner.references()?;
        let mut branches = vec![];
        for branch in refs.local_branches()? {
            let branch = branch.map(|branch| branch.name().shorten().to_string());
            match branch {
                Ok(branch) => branches.push(branch),
                Err(e) => tracing::error!("Failed to list branch: {e}"),
            }
        }

        Ok(branches)
    }
}

mod imp {
    use crate::GitRepository;

    use std::path::PathBuf;

    impl GitRepository {
        pub fn init_bare(base: PathBuf, name: &str) -> anyhow::Result<GitRepository> {
            let path = base.join(format!("{name}.git"));
            tracing::debug!("Initializing repository {:?}", path);
            let repository = git_repository::init_bare(path)?;
            let hook_path = repository.path().join("hooks");
            std::os::unix::fs::symlink(
                "/home/git/hooks/post-receive",
                hook_path.join("post-receive"),
            )?;

            Ok(GitRepository { inner: repository })
        }
    }

    #[cfg(test)]
    mod test {
        use crate::GitRepository;
        use cmd_lib::run_cmd;
        use sealed_test::prelude::*;
        use speculoos::prelude::*;

        use std::path::PathBuf;

        #[sealed_test]
        fn should_init_bare() -> anyhow::Result<()> {
            let repository = GitRepository::init_bare(PathBuf::from("."), "repo")?;
            assert_that!(repository.inner.path().to_string_lossy()).ends_with("repo.git");
            Ok(())
        }

        #[sealed_test]
        fn should_list_branches() -> anyhow::Result<()> {
            run_cmd!(
                git init;
                git commit --allow-empty -m "First commit";
                git checkout -b A;
                git checkout -b B;

            )?;

            let repo = GitRepository {
                inner: git_repository::open(".")?,
            };
            let branches = repo.list_branches();

            assert_that!(branches).is_ok().contains_all_of(&[
                &"master".to_string(),
                &"A".to_string(),
                &"B".to_string(),
            ]);

            Ok(())
        }
    }
}
