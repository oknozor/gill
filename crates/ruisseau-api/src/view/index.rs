use crate::oauth::Oauth2User;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::response::IntoResponse;
use axum::Extension;
use ruisseau_db::user::User;
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
    let email = user.map(|user| user.email);
    let email = match email {
        Some(email) => User::by_email(&email, &db).await.ok(),
        None => None,
    }
    .map(|user| user.username);
    let template = LandingPageTemplate { user: email };
    HtmlTemplate(template)
}
