use git_repository::init::Error;
use git_repository::Repository;
use std::path::PathBuf;

pub fn bare(base: PathBuf, name: &str) -> Result<Repository, Error> {
    let path = base.join(format!("{name}.git"));
    git_repository::init_bare(path)
}

#[cfg(test)]
mod test {
    use crate::init::bare;
    use sealed_test::prelude::*;
    use std::path::PathBuf;

    #[sealed_test]
    fn should_init_bare() {
        let repository = bare(PathBuf::from("."), "repo").unwrap();
        assert!(repository.path().ends_with("repo.git"));
    }
}
