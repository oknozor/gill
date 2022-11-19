use crate::oauth::Oauth2User;
use crate::route::user::User;
use crate::SETTINGS;
use axum::{
    http,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use serde_json::Value;
use sqlx::PgPool;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| reqwest::Client::new());

pub async fn auth<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = match auth_header {
        Some(auth_header) => auth_header,
        None => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    tracing::debug!("Got bearer {auth_header}");

    match user_info(auth_header).await {
        Ok(current_user) => {
            let pool = req
                .extensions()
                .get::<PgPool>()
                .expect("No database connection");
            match User::by_email(&current_user.email, &pool).await {
                Err(err) => {
                    tracing::error!("Error fetching current user '{}': {err}", current_user.email);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                Ok(user) => {
                    req.extensions_mut().insert(user);
                    Ok(next.run(req).await)
                }
            }
        }
        Err(err) => {
            tracing::error!("User info failed {err}");
            Err(StatusCode::FORBIDDEN)
        }
    }
}

async fn user_info(bearer: &str) -> eyre::Result<Oauth2User> {
    let value: Value = CLIENT
        .get(&SETTINGS.user_info_url)
        .header("Authorization", bearer)
        .send()
        .await?
        .json()
        .await?;

    tracing::debug!("UserInfo response: {value:?}");

    serde_json::from_value(value)
        .map_err(Into::into)
}
