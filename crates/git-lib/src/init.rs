use std::path::PathBuf;
use git_repository::Repository;
use git_repository::init::Error;

pub fn bare(base: PathBuf, name: &str) -> Result<Repository, Error> {
    let path = base.join(format!("{name}.git"));
    git_repository::init_bare(path)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use sealed_test::prelude::*;
    use crate::init::bare;

    #[sealed_test]
    fn should_init_bare() {
        let repository = bare(PathBuf::from("."), "repo").unwrap();
        assert!(repository.path().ends_with("repo.git"));
    }
}
