use crate::oauth::Oauth2User;
use crate::view::{get_connected_user_username, HtmlTemplate};
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;

use crate::error::AppError;
use crate::view::dto::RepositoryDto;
use gill_db::activity::Activity;
use gill_db::repository::RepositoryDigest;
use sqlx::PgPool;

pub struct ActivityDto {}

#[derive(Template)]
#[template(path = "index.html")]
struct LandingPageTemplate {
    user: Option<String>,
    local_repositories: Vec<RepositoryDto>,
    all_repositories: Vec<RepositoryDto>,
    activities: Vec<Activity>,
}

pub async fn index(
    Extension(db): Extension<PgPool>,
    user: Option<Oauth2User>,
) -> Result<impl IntoResponse, AppError> {
    let username = get_connected_user_username(&db, user).await;
    let local_repositories = RepositoryDigest::all_local(10, 0, &db).await?;
    let local_repositories = local_repositories
        .into_iter()
        .map(RepositoryDto::from)
        .collect();

    let template = LandingPageTemplate {
        user: username,
        local_repositories,
        all_repositories: vec![],
        activities: vec![],
    };

    Ok(HtmlTemplate(template))
}
