use crate::oauth::Oauth2User;
use crate::view::{get_connected_user_username, HtmlTemplate};
use askama::Template;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use gill_db::user::User;
use http::Response;
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "user/profile.html")]
pub struct UserProfileTemplate {
    user: Option<String>,
}

#[derive(Debug)]
pub struct RepositoryDo {
    name: String,
    is_default: bool,
    is_current: bool,
}

pub async fn get_profile(
    connected_user: Option<Oauth2User>,
    Path(user): Path<String>,
    Extension(db): Extension<PgPool>,
) -> Result<HtmlTemplate<UserProfileTemplate>, crate::error::AppError> {
    let username = get_connected_user_username(&db, connected_user).await;
    let user = User::by_user_name(&user, &db).await?;
    let repositories = user.list_repositories(&db).await?;

    let template = UserProfileTemplate { user: username };
    Ok(HtmlTemplate(template))
}
