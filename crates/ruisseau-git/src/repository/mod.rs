use std::fs;
use std::path::PathBuf;
use git_repository::Repository;
use git_repository::init::Error;


pub fn init_bare(namespace: &str, name: &str) -> Result<Repository, Error> {
    let path = PathBuf::from(namespace);
    if !path.exists() {
        fs::create_dir(&path).expect("Failed to create dir");
    }

    private::init_bare(path, name)
}

mod private {
    use std::path::PathBuf;
    use git_repository::Repository;
    use git_repository::init::Error;

    pub fn init_bare(base: PathBuf, name: &str) -> Result<Repository, Error> {
        let path = base.join(format!("{name}.git"));
        git_repository::init_bare(path)
    }

    #[cfg(test)]
    mod test {
        use sealed_test::prelude::*;
        use std::path::PathBuf;
        use super::init_bare;

        #[sealed_test]
        fn should_init_bare() {
            let repository = init_bare(PathBuf::from("."), "repo").unwrap();
            assert!(repository.path().ends_with("repo.git"));
        }
    }
}

