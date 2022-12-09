use crate::fixtures::{ALICE_ID, GILL_REPO_ID, LINUX_REPO_ID};
use gill_db::repository::{Branch, CreateRepository, OwnedRepository, Repository};
use speculoos::prelude::*;
use sqlx::PgPool;

#[sqlx::test(fixtures("base"))]
async fn should_create_repository(db: PgPool) {
    let repository = CreateRepository {
        name: "myrepo".to_string(),
    };

    let other_repo = CreateRepository {
        name: "myotherrepo".to_string(),
    };

    let res = Repository::create(ALICE_ID, &repository, &db).await;
    assert_that!(res).is_ok();

    let res = Repository::create(ALICE_ID, &other_repo, &db).await;
    assert_that!(res).is_ok();
}

#[sqlx::test(fixtures("base"))]
async fn should_get_repository_by_namespace(db: PgPool) {
    let linux = Repository::by_namespace("alice", "linux", &db).await;

    assert_that!(linux).is_ok().is_equal_to(Repository {
        id: LINUX_REPO_ID,
        name: "linux".to_string(),
        description: None,
        private: false,
        owner_id: ALICE_ID,
    });
}

#[sqlx::test(fixtures("base"))]
async fn should_list_repositories(db: PgPool) {
    let repositories = Repository::list(10, 0, &db).await;
    assert_that!(repositories).is_ok().contains_all_of(&[
        &OwnedRepository {
            id: GILL_REPO_ID,
            owner_id: 1,
            name: "gill".to_string(),
            owner_name: "okno".to_string(),
            description: None,
            private: false,
        },
        &OwnedRepository {
            id: LINUX_REPO_ID,
            owner_id: 0,
            name: "linux".to_string(),
            owner_name: "alice".to_string(),
            description: None,
            private: false,
        },
    ]);
}

#[sqlx::test(fixtures("base"))]
async fn list_branch(db: PgPool) {
    let repository = fixtures::gill_repository();

    let res = repository.list_branches(5, 0, &db).await;
    assert_that!(res).is_ok().contains_all_of(&[
        &Branch {
            name: "main".to_string(),
            repository_id: GILL_REPO_ID,
            is_default: true,
        },
        &Branch {
            name: "feature".to_string(),
            repository_id: GILL_REPO_ID,
            is_default: false,
        },
        &Branch {
            name: "fix".to_string(),
            repository_id: GILL_REPO_ID,
            is_default: false,
        },
    ]);
}

#[sqlx::test(fixtures("base"))]
async fn create_default_branch(db: PgPool) -> anyhow::Result<()> {
    let repository = fixtures::linux_kernel_repository();

    let res = repository.set_default_branch("main", &db).await;
    assert_that!(res).is_ok();

    let branches = repository.list_branches(5, 0, &db).await?;

    assert_that!(branches).has_length(1);
    assert_that!(branches).contains(Branch {
        name: "main".to_string(),
        repository_id: LINUX_REPO_ID,
        is_default: true,
    });
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn change_default_branch(db: PgPool) -> anyhow::Result<()> {
    let repository = fixtures::gill_repository();

    let res = repository.set_default_branch("feature", &db).await;
    assert_that!(res).is_ok();

    let branches = repository.list_branches(5, 0, &db).await?;

    let default_branch: Vec<&String> = branches
        .iter()
        .filter(|b| b.is_default)
        .map(|b| &b.name)
        .collect();

    assert_that!(default_branch).has_length(1);
    assert_that!(default_branch).contains(&"feature".to_string());
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn get_default_branch(db: PgPool) -> anyhow::Result<()> {
    let repository = fixtures::gill_repository();
    let res = repository.get_default_branch(&db).await;
    assert_that!(res).is_some().is_equal_to(Branch {
        name: "main".to_string(),
        repository_id: GILL_REPO_ID,
        is_default: true,
    });

    Ok(())
}

mod fixtures {
    use gill_db::repository::Repository;

    pub const ALICE_ID: i32 = 0;
    pub const GILL_REPO_ID: i32 = 2;
    pub const LINUX_REPO_ID: i32 = 1;

    // This fixture repo has branches already
    // see: fixture/base.sql
    pub fn gill_repository() -> Repository {
        Repository {
            id: GILL_REPO_ID,
            name: "gill".to_string(),
            description: None,
            private: false,
            owner_id: 1,
        }
    }

    // A repo without branches
    pub fn linux_kernel_repository() -> Repository {
        Repository {
            id: LINUX_REPO_ID,
            name: "linux".to_string(),
            description: None,
            private: false,
            owner_id: 0,
        }
    }
}
