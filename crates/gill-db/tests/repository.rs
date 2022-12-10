use crate::fixtures::{create_repository, GILL_REPO_ID, LINUX_REPO_ID};
use gill_db::repository::{Branch, Repository};
use speculoos::prelude::*;
use sqlx::PgPool;

#[sqlx::test(fixtures("base"))]
async fn should_create_repository(db: PgPool) {
    let repo = create_repository();

    let res = Repository::create(&repo, &db).await;
    assert_that!(res).is_ok();
}

#[sqlx::test(fixtures("base"))]
async fn should_get_repository_by_namespace(db: PgPool) {
    let linux = Repository::by_namespace("alice", "linux", &db).await;

    assert_that!(linux).is_ok().is_equal_to(Repository {
        id: LINUX_REPO_ID,
        activity_pub_id: "".to_string(),
        name: "linux".to_string(),
        private: false,
        inbox_url: "".to_string(),
        outbox_url: "".to_string(),
        followers_url: "".to_string(),
        attributed_to: "".to_string(),
        clone_uri: "".to_string(),
        public_key: "".to_string(),
        private_key: None,
        published: Default::default(),
        ticket_tracked_by: "".to_string(),
        send_patches_to: "".to_string(),
        domain: "".to_string(),
        summary: None,
        is_local: false,
    });
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
    use gill_db::repository::{CreateRepository, Repository};

    pub const GILL_REPO_ID: i32 = 2;
    pub const LINUX_REPO_ID: i32 = 1;

    // This fixture repo has branches already
    // see: fixture/base.sql
    pub fn gill_repository() -> Repository {
        let id = "https://instance.org/users/alice/repositories/gill".to_string();
        let user_id = "https://instance.org/users/alice".to_string();
        Repository {
            id: GILL_REPO_ID,
            activity_pub_id: id.clone(),
            name: "gill".to_string(),
            private: false,
            inbox_url: format!("{id}/inbox"),
            outbox_url: format!("{id}/outbox"),
            followers_url: format!("{id}/followers"),
            attributed_to: user_id.clone(),
            clone_uri: user_id.clone(),
            public_key: "12345".to_string(),
            private_key: None,
            published: Default::default(),
            ticket_tracked_by: user_id.clone(),
            send_patches_to: user_id,
            domain: "instance.org".to_string(),
            summary: None,
            is_local: false,
        }
    }

    // A repo without branches
    pub fn linux_kernel_repository() -> Repository {
        let id = "https://okno.org/users/okno/repositories/linux".to_string();
        let user_id = "https://okno.org/users/okno".to_string();

        Repository {
            id: LINUX_REPO_ID,
            activity_pub_id: "".to_string(),
            name: "linux".to_string(),
            private: false,
            inbox_url: format!("{id}/inbox"),
            outbox_url: format!("{id}/outbox"),
            followers_url: format!("{id}/followers"),
            attributed_to: user_id.clone(),
            clone_uri: user_id.clone(),
            public_key: "12345".to_string(),
            private_key: None,
            published: Default::default(),
            ticket_tracked_by: user_id.clone(),
            send_patches_to: user_id,
            domain: "instance.org".to_string(),
            summary: None,
            is_local: false,
        }
    }

    // A repo without branches
    pub fn create_repository() -> CreateRepository {
        let id = "https://okno.org/users/john/repositories/myrepo".to_string();
        let user_id = "https://okno.org/users/john".to_string();

        CreateRepository {
            activity_pub_id: "".to_string(),
            name: "myrepo".to_string(),
            private: false,
            inbox_url: format!("{id}/inbox"),
            outbox_url: format!("{id}/outbox"),
            followers_url: format!("{id}/followers"),
            attributed_to: user_id.clone(),
            clone_uri: user_id.clone(),
            public_key: "12345".to_string(),
            private_key: None,
            ticket_tracked_by: user_id.clone(),
            send_patches_to: user_id,
            domain: "instance.org".to_string(),
            summary: None,
            is_local: false,
        }
    }
}
