use askama::Template;
use axum::Extension;
use axum::response::IntoResponse;
use sqlx::PgPool;
use crate::app::HtmlTemplate;
use crate::route::repository::{OwnedRepository, Repository};

#[derive(Template)]
#[template(path = "repositories-list.html")]
struct RepositoryListTemplate {
    repositories: Vec<OwnedRepository>
}


pub async fn list(
    Extension(pool): Extension<PgPool>
) -> impl IntoResponse {
    let repositories = Repository::list(20, 0, &pool).await.unwrap();
    let template = RepositoryListTemplate { repositories };
    HtmlTemplate(template)
}

