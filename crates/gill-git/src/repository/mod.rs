use git_repository::Repository;
use std::fs;
use std::path::PathBuf;

const REPO_DIR: &str = "/home/git";

pub fn init_bare(namespace: &str, name: &str) -> anyhow::Result<Repository> {
    let path = PathBuf::from(REPO_DIR).join(namespace);

    if !path.exists() {
        fs::create_dir(&path).expect("Failed to create dir");
    }

    imp::init_bare(path, name)
}

pub fn list_branch(namespace: &str, name: &str) -> anyhow::Result<Vec<String>> {
    let name = format!("{name}.git");
    let path = PathBuf::from(REPO_DIR).join(namespace).join(name);
    imp::list_branches(path)
}

mod imp {

    use git_repository::Repository;
    use std::path::PathBuf;

    pub fn init_bare(base: PathBuf, name: &str) -> anyhow::Result<Repository> {
        let path = base.join(format!("{name}.git"));
        tracing::debug!("Initializing repository {:?}", path);
        let repository = git_repository::init_bare(path)?;
        let hook_path = repository.path().join("hooks");
        std::os::unix::fs::symlink(
            "/home/git/hooks/post-receive",
            hook_path.join("post-receive"),
        )?;
        Ok(repository)
    }

    pub fn list_branches(path: PathBuf) -> anyhow::Result<Vec<String>> {
        let repo = git_repository::open(path)?;
        let refs = repo.references()?;
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

    #[cfg(test)]
    mod test {
        use super::init_bare;
        use crate::repository::imp::list_branches;
        use sealed_test::prelude::*;
        use speculoos::prelude::*;
        use std::env;
        use std::path::PathBuf;

        #[sealed_test]
        fn should_init_bare() -> anyhow::Result<()> {
            let repository = init_bare(PathBuf::from("."), "repo")?;
            assert_that!(repository.path().to_string_lossy()).ends_with("repo.git");
            Ok(())
        }

        #[test]
        fn should_list_branches() -> anyhow::Result<()> {
            let current = env::var("CARGO_MANIFEST_DIR")?;
            let mut current = PathBuf::from(current);
            // FIXME: we need some test suite tools like in cocogitto
            current.pop();
            current.pop();
            let branches = list_branches(current);
            assert_that!(branches)
                .is_ok()
                .contains_all_of(&[&"main".to_string(), &"feat-test".to_string()]);

            Ok(())
        }
    }
}
