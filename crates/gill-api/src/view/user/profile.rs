use crate::oauth::Oauth2User;
use crate::view::{get_connected_user_username, HtmlTemplate};
use askama::Template;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "user/profile.html")]
pub struct UserProfileTemplate {
    user: Option<String>,
}

pub async fn get_profile(
    connected_user: Option<Oauth2User>,
    Path(_user): Path<String>,
    Extension(db): Extension<PgPool>,
) -> impl IntoResponse {
    let username = get_connected_user_username(&db, connected_user).await;
    let template = UserProfileTemplate { user: username };
    HtmlTemplate(template)
}
