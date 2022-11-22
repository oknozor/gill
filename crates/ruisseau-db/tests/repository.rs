use crate::fixtures::{ALICE_ID, LINUX_REPO_ID, RUISSEAU_REPO_ID};
use ruisseau_db::repository::{Branch, InitRepository, OwnedRepository, Repository};
use speculoos::prelude::*;
use sqlx::PgPool;

#[sqlx::test(fixtures("base"))]
async fn should_create_repository(db: PgPool) {
    let repository = InitRepository {
        name: "myrepo".to_string(),
    };

    let other_repo = InitRepository {
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
        owner_id: ALICE_ID,
    });
}

#[sqlx::test(fixtures("base"))]
async fn should_list_repositories(db: PgPool) {
    let repositories = Repository::list(10, 0, &db).await;
    assert_that!(repositories).is_ok().contains_all_of(&[
        &OwnedRepository {
            id: RUISSEAU_REPO_ID,
            owner_id: 1,
            name: "ruisseau".to_string(),
            owner_name: "okno".to_string(),
        },
        &OwnedRepository {
            id: LINUX_REPO_ID,
            owner_id: 0,
            name: "linux".to_string(),
            owner_name: "alice".to_string(),
        },
    ]);
}

#[sqlx::test(fixtures("base"))]
async fn list_branch(db: PgPool) {
    let repository = fixtures::ruisseau_repository();

    let res = repository.list_branches(5, 0, &db).await;
    assert_that!(res).is_ok().contains_all_of(&[
        &Branch {
            name: "main".to_string(),
            repository_id: RUISSEAU_REPO_ID,
            is_default: true,
        },
        &Branch {
            name: "feature".to_string(),
            repository_id: RUISSEAU_REPO_ID,
            is_default: false,
        },
        &Branch {
            name: "fix".to_string(),
            repository_id: RUISSEAU_REPO_ID,
            is_default: false,
        },
    ]);
}

#[sqlx::test(fixtures("base"))]
async fn create_default_branch(db: PgPool) -> eyre::Result<()> {
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
async fn change_default_branch(db: PgPool) -> eyre::Result<()> {
    let repository = fixtures::ruisseau_repository();

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

mod fixtures {
    use ruisseau_db::repository::Repository;

    pub const ALICE_ID: i32 = 0;
    pub const RUISSEAU_REPO_ID: i32 = 2;
    pub const LINUX_REPO_ID: i32 = 1;

    // This fixture repo has branches already
    // see: fixture/base.sql
    pub fn ruisseau_repository() -> Repository {
        Repository {
            id: RUISSEAU_REPO_ID,
            name: "ruisseau".to_string(),
            owner_id: 1,
        }
    }

    // A repo without branches
    pub fn linux_kernel_repository() -> Repository {
        Repository {
            id: LINUX_REPO_ID,
            name: "linux".to_string(),
            owner_id: 0,
        }
    }
}
