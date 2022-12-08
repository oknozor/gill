use anyhow::anyhow;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::get;
use http::StatusCode;
use webfinger::{Link, Prefix, ResolverError, Webfinger, WebfingerError};
use gill_settings::SETTINGS;
use crate::error::AppError;
use crate::oauth::AppState;
use serde::{Deserialize};

#[derive(Deserialize)]
pub struct WebFingerQuery {
    resource: String,
}

impl WebFingerQuery {
    fn parse(&self) -> Option<(String, String)>{
        self.resource.split_once(':')
            .and_then(|(prefix, res)| {
                res.split_once('@')
            })
            .map(|(user, domain)| (user.to_string(), domain.to_string()))
    }
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/.wellknown/webfinger", get(webfinger))
        .with_state(app_state)
}

async fn webfinger(Query(query): Query<WebFingerQuery>) -> impl IntoResponse {
    let (user, domain) = query.parse().unwrap();
    if domain == SETTINGS.domain {
        Ok(Json(Webfinger {
            subject: query.resource,
            aliases: vec![
                format!("http://{}/@{}", domain, user),
                format!("http://{}/apub/users/{}", domain, user),
            ],
            links: vec![
                Link {
                    rel: "self".to_string(),
                    href: Some(format!("http://{}/@{}", domain, user)),
                    template: None,
                    mime_type: Some("application/activity+json".to_string()),
                }
            ],
        }))
    } else {
        Err(AppError::from(anyhow!("Webfinger acct not found")))
    }
}
