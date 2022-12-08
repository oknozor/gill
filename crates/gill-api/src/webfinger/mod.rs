use crate::error::AppError;

use anyhow::anyhow;
use axum::extract::Query;
use axum::response::IntoResponse;

use axum::Json;
use gill_settings::SETTINGS;

use serde::Deserialize;
use webfinger::{Link, Webfinger};

#[derive(Deserialize)]
pub struct WebFingerQuery {
    resource: String,
}

impl WebFingerQuery {
    fn parse(&self) -> Option<(String, String)> {
        self.resource
            .split_once(':')
            .and_then(|(_prefix, res)| res.split_once('@'))
            .map(|(user, domain)| (user.to_string(), domain.to_string()))
    }
}

pub async fn webfinger(Query(query): Query<WebFingerQuery>) -> impl IntoResponse {
    let (user, domain) = query.parse().unwrap();
    if domain == SETTINGS.domain {
        Ok(Json(Webfinger {
            subject: query.resource,
            aliases: vec![
                format!("http://{}/@{}", domain, user),
                format!("http://{}/apub/users/{}", domain, user),
            ],
            links: vec![Link {
                rel: "self".to_string(),
                href: Some(format!("http://{}/apub/users/{}", domain, user)),
                template: None,
                mime_type: Some("application/activity+json".to_string()),
            }],
        }))
    } else {
        Err(AppError::from(anyhow!("Webfinger acct not found")))
    }
}
