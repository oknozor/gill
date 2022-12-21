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
pub mod syntax;
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
