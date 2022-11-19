use git_repository::init::Error;
use git_repository::Repository;
use std::fs;
use std::path::PathBuf;

pub fn init_bare(base_path: &PathBuf, namespace: &str, name: &str) -> Result<Repository, Error> {
    let path = base_path.join(namespace);

    if !path.exists() {
        fs::create_dir(&path).expect("Failed to create dir");
    }

    private::init_bare(path, name)
}

pub fn list_branch(base_path: &PathBuf, namespace: &str, name: &str) -> eyre::Result<Vec<String>> {
    let name = format!("{name}.git");
    let path = PathBuf::from(base_path)
        .join(namespace)
        .join(name);

    println!("{:?}", path);

    private::list_branches(path)
}


mod private {
    use git_repository::init::Error;
    use git_repository::Repository;
    use std::path::PathBuf;

    pub fn init_bare(base: PathBuf, name: &str) -> Result<Repository, Error> {
        let path = base.join(format!("{name}.git"));
        tracing::debug!("Initializing repository {:?}", path);
        git_repository::init_bare(path)
    }

    pub fn list_branches(path: PathBuf) -> eyre::Result<Vec<String>> {
        let repo = git_repository::open(path)?;
        let refs = repo.references()?;
        let mut branches = vec![];
        for branch in refs.local_branches()? {
            let branch = branch.map(|branch| branch.name().shorten().to_string());
            if let Ok(branch) = branch {
                branches.push(branch);
            }
        };

        Ok(branches)
    }


    #[cfg(test)]
    mod test {
        use std::env;
        use super::init_bare;
        use sealed_test::prelude::*;
        use speculoos::prelude::*;
        use std::path::PathBuf;
        use crate::repository::private::list_branches;

        #[sealed_test]
        fn should_init_bare() -> eyre::Result<()> {
            let repository = init_bare(PathBuf::from("."), "repo")?;
            assert_that!(repository.path().to_string_lossy()).ends_with("repo.git");
            Ok(())
        }

        #[test]
        fn should_list_branches() -> eyre::Result<()> {
            let current = env::var("CARGO_MANIFEST_DIR")?;
            let mut current = PathBuf::from(current);
            // FIXME: we need some test suite tools like in cocogitto
            current.pop();
            current.pop();
            let branches = list_branches(current);
            assert_that!(branches)
                .is_ok()
                .contains_all_of(&[
                    &"main".to_string(),
                    &"feat-test".to_string(),
                ]);

            Ok(())
        }
    }
}
