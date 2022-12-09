use crate::oauth::Oauth2User;
use crate::view::{get_connected_user_username, HtmlTemplate};
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;

use crate::error::AppError;
use gill_db::activity::Activity;
use gill_db::repository::RepositoryDigest;
use sqlx::PgPool;

pub struct ActivityDto {}

pub struct LocalRepositoryDto {
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub star_count: u32,
    pub fork_count: u32,
}

pub struct RepositoryDto {
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub star_count: u32,
    pub fork_count: u32,
}

impl From<RepositoryDigest> for RepositoryDto {
    fn from(repo: RepositoryDigest) -> Self {
        RepositoryDto {
            owner: repo.owner,
            name: repo.name,
            description: repo.summary,
            star_count: repo.star_count.map(|c| c as u32).unwrap_or(0),
            fork_count: repo.fork_count.map(|c| c as u32).unwrap_or(0),
        }
    }
}

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
