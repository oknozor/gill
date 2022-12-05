use crate::oauth::Oauth2User;
use crate::view::{get_connected_user_username, HtmlTemplate};
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;
use gill_db::repository::{OwnedRepository, Repository};
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "repositories-list.html")]
struct RepositoryListTemplate {
    repositories: Vec<OwnedRepository>,
    user: Option<String>,
}

pub async fn list(user: Option<Oauth2User>, Extension(db): Extension<PgPool>) -> impl IntoResponse {
    let connected_username = get_connected_user_username(&db, user).await;
    let repositories = Repository::list(20, 0, &db).await.unwrap();
    let template = RepositoryListTemplate {
        repositories,
        user: connected_username,
    };
    HtmlTemplate(template)
}
