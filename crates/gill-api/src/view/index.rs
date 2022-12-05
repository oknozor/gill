use crate::oauth::Oauth2User;
use crate::view::{get_connected_user_username, HtmlTemplate};
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;

use sqlx::PgPool;

#[derive(Template)]
#[template(path = "index.html")]
struct LandingPageTemplate {
    user: Option<String>,
}

pub async fn index(
    Extension(db): Extension<PgPool>,
    user: Option<Oauth2User>,
) -> impl IntoResponse {
    let username = get_connected_user_username(&db, user).await;
    let template = LandingPageTemplate { user: username };
    HtmlTemplate(template)
}
