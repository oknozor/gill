use crate::oauth::Oauth2User;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;

use crate::domain::repository::digest::RepositoryDigest;
use crate::error::AppError;
use crate::get_connected_user_username;
use crate::view::dto::{FederatedRepositoryDto, RepositoryDto};
use sqlx::PgPool;

pub struct ActivityDto {}

#[derive(Template)]
#[template(path = "index.html")]
struct LandingPageTemplate {
    user: Option<String>,
    local_repositories: Vec<RepositoryDto>,
    federated_repositories: Vec<FederatedRepositoryDto>,
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

    let federated_repositories = RepositoryDigest::all_federated(10, 0, &db).await?;
    let federated_repositories = federated_repositories
        .into_iter()
        .map(FederatedRepositoryDto::from)
        .collect();

    let template = LandingPageTemplate {
        user: username,
        local_repositories,
        federated_repositories,
    };

    Ok(HtmlTemplate(template))
}
