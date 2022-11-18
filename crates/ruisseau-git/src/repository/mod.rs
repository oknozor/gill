use git_repository::init::Error;
use git_repository::Repository;
use std::fs;
use std::path::PathBuf;

pub fn init_bare(namespace: &str, name: &str) -> Result<Repository, Error> {
    let path = PathBuf::from(namespace);
    if !path.exists() {
        fs::create_dir(&path).expect("Failed to create dir");
    }

    private::init_bare(path, name)
}

mod private {
    use git_repository::init::Error;
    use git_repository::Repository;
    use std::path::PathBuf;

    pub fn init_bare(base: PathBuf, name: &str) -> Result<Repository, Error> {
        let path = base.join(format!("{name}.git"));
        git_repository::init_bare(path)
    }

    #[cfg(test)]
    mod test {
        use super::init_bare;
        use sealed_test::prelude::*;
        use std::path::PathBuf;

        #[sealed_test]
        fn should_init_bare() {
            let repository = init_bare(PathBuf::from("."), "repo").unwrap();
            assert!(repository.path().ends_with("repo.git"));
        }
    }
}
