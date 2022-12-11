use crate::error::AppError;
use crate::oauth::Oauth2User;
use crate::view::{get_connected_user_username, HtmlTemplate};
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;
use gill_db::user::User;

use sqlx::PgPool;

#[derive(Template)]
#[template(path = "user/settings.html")]
pub struct UserSettingsTemplate {
    user: Option<String>,
}

pub async fn settings(
    connected_user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let Some(user) = get_connected_user_username(&db, connected_user).await else {
        return Err(AppError::Unauthorized);
    };

    let user = User::by_user_name(&user, &db).await?;

    Ok(HtmlTemplate(UserSettingsTemplate {
        user: Some(user.username),
    }))
}
