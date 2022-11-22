use crate::view::HtmlTemplate;
use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "login-callback.html")]
struct LoginCallBackTemplate;

pub async fn callback() -> impl IntoResponse {
    HtmlTemplate(LoginCallBackTemplate)
}
