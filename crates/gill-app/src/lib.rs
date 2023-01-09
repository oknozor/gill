extern crate core;

use crate::oauth::Oauth2User;
use gill_db::user::User;
use sqlx::PgPool;

pub mod api;
pub mod apub;
pub mod domain;
pub mod error;
pub mod instance;
pub mod oauth;
pub mod state;
pub mod view;
pub mod webfinger;

async fn get_connected_user_username(db: &PgPool, user: Option<Oauth2User>) -> Option<String> {
    get_connected_user(db, user).await.map(|user| user.username)
}

async fn get_connected_user(db: &PgPool, user: Option<Oauth2User>) -> Option<User> {
    let email = user.map(|user| user.email);
    match email {
        Some(email) => User::by_email(&email, db).await.ok(),
        None => None,
    }
}

#[cfg(test)]
mod test {
    use archunit_rs::{ExludeModules, Modules};
    use archunit_rs::rule::{ArchRuleBuilder, CheckRule};

    #[test]
    fn only_domain_should_access_database() {
        Modules::that(ExludeModules::cfg_test())
            .does_not_reside_in_a_module("gill_app::domain*")
            .should()
            .only_have_dependency_module()
            .that()
            .does_not_have_simple_name("gill_db")
            .check();
    }
}
