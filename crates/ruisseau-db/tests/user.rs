use ruisseau_db::user::{CreateUser, User};
use speculoos::prelude::*;
use sqlx::PgPool;

const ALICE_ID: i32 = 0;

#[sqlx::test]
async fn should_create_user(db: PgPool) {
    let bob = CreateUser {
        username: "Bob".to_string(),
        email: "bob@ruisseau.org".to_string(),
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
        email: "alice@wonder.land".to_string(),
    });
}

#[sqlx::test(fixtures("base"))]
async fn should_add_ssh_key(db: PgPool) {
    let res = User::add_ssh_key(ALICE_ID, "my-ssh-key", &db).await;

    assert_that!(res).is_ok();
}
