mod follower;

use ruisseau_db::user::{CreateUser, User};
use speculoos::prelude::*;
use sqlx::PgPool;

const ALICE_ID: i32 = 0;

#[sqlx::test]
async fn should_create_user(db: PgPool) {
    let bob = CreateUser {
        username: "Bob".to_string(),
        email: "bob@ruisseau.org".to_string(),
        private_key: Some("private_key".to_string()),
        public_key: "public_key".to_string(),
        is_local: true,
        followers_url: "https://myinstance.org/bob/followers/".to_string(),
        outbox_url: "https://myinstance.org/bob/outbox/".to_string(),
        inbox_url: "https://myinstance.org/bob/inbox/".to_string(),
        activity_pub_id: "https://myinstance.org/bob".to_string(),
        domain: "myinstance.org".to_string(),
    };

    let res = User::create(bob, &db).await;

    assert_that!(res).is_ok();
}

#[sqlx::test(fixtures("base"))]
async fn should_get_user_by_email(db: PgPool) {
    let alice = User::by_email("alice@wonder.land", &db).await;

    assert_that!(alice).is_ok().is_equal_to(User {
        id: ALICE_ID,
        username: "alice".to_string(),
        domain: "myinstance.org".to_string(),
        email: "alice@wonder.land".to_string(),
        public_key: "public_key".to_string(),
        private_key: Some("private_key".to_string()),
        inbox_url: "https://myinstance.org/alice/inbox/".to_string(),
        outbox_url: "https://myinstance.org/alice/outbox/".to_string(),
        followers_url: "https://myinstance.org/alice/followsers/".to_string(),
        activity_pub_id: "https://myinstance.org/alice".to_string(),
        is_local: true,
    });
}

#[sqlx::test(fixtures("base"))]
async fn should_get_user_by_username(db: PgPool) {
    let alice = User::by_user_name("alice", &db).await;

    assert_that!(alice).is_ok().is_equal_to(User {
        id: ALICE_ID,
        username: "alice".to_string(),
        domain: "myinstance.org".to_string(),
        email: "alice@wonder.land".to_string(),
        public_key: "public_key".to_string(),
        private_key: Some("private_key".to_string()),
        inbox_url: "https://myinstance.org/alice/inbox/".to_string(),
        outbox_url: "https://myinstance.org/alice/outbox/".to_string(),
        followers_url: "https://myinstance.org/alice/followsers/".to_string(),
        activity_pub_id: "https://myinstance.org/alice".to_string(),
        is_local: true,
    });
}

#[sqlx::test(fixtures("base"))]
async fn should_get_user_by_activity_pub_id(db: PgPool) {
    let alice = User::by_activity_pub_id("https://myinstance.org/alice", &db).await;

    assert_that!(alice).is_ok().is_equal_to(User {
        id: ALICE_ID,
        username: "alice".to_string(),
        domain: "myinstance.org".to_string(),
        email: "alice@wonder.land".to_string(),
        public_key: "public_key".to_string(),
        private_key: Some("private_key".to_string()),
        inbox_url: "https://myinstance.org/alice/inbox/".to_string(),
        outbox_url: "https://myinstance.org/alice/outbox/".to_string(),
        followers_url: "https://myinstance.org/alice/followsers/".to_string(),
        activity_pub_id: "https://myinstance.org/alice".to_string(),
        is_local: true,
    });
}

#[sqlx::test(fixtures("base"))]
async fn should_add_ssh_key(db: PgPool) {
    let res = User::add_ssh_key(ALICE_ID, "my-ssh-key", &db).await;

    assert_that!(res).is_ok();
}
