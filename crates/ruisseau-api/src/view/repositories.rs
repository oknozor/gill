use crate::view::HtmlTemplate;
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;
use ruisseau_db::repository::{OwnedRepository, Repository};
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "repositories-list.html")]
struct RepositoryListTemplate {
    repositories: Vec<OwnedRepository>,
}

pub async fn list(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let repositories = Repository::list(20, 0, &pool).await.unwrap();
    let template = RepositoryListTemplate { repositories };
    HtmlTemplate(template)
}
