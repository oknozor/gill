use crate::oauth::Oauth2User;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::{Path, Query};

use axum::Extension;

use crate::view::dto::RepositoryDto;

use crate::domain::user::User;
use crate::get_connected_user_username;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct UserProfileQuery {
    #[serde(default)]
    tab: Tab,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tab {
    Profile,
    Repositories,
    Stars,
}

impl Default for Tab {
    fn default() -> Self {
        Self::Repositories
    }
}

#[derive(Template)]
#[template(path = "user/profile.html")]
pub struct UserPageTemplate {
    profile_username: String,
    user: Option<String>,
    repositories: Vec<RepositoryDto>,
    stars: Vec<RepositoryDto>,
    tab: Tab,
}

pub async fn user_view(
    connected_user: Option<Oauth2User>,
    Path(user): Path<String>,
    Query(page): Query<UserProfileQuery>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<UserPageTemplate>, crate::error::AppError> {
    let profile_username = user;
    let user = User::by_name(&profile_username, &db).await?;

    let repositories = user
        .list_repositories(20, 0, &db)
        .await?
        .into_iter()
        .map(RepositoryDto::from)
        .collect();

    let stars = user
        .starred_repositories(20, 0, &db)
        .await?
        .into_iter()
        .map(RepositoryDto::from)
        .collect();

    let username = get_connected_user_username(&db, connected_user).await;

    let template = UserPageTemplate {
        profile_username,
        user: username,
        repositories,
        stars,
        tab: page.tab,
    };

    Ok(HtmlTemplate(template))
}
