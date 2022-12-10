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

struct WebFingerAcct {
    user: String,
    domain: String,
    repository: Option<String>,
}

impl WebFingerQuery {
    fn parse(&self) -> Option<WebFingerAcct> {
        let Some((user, domain)) = self.resource
            .split_once(':')
            .and_then(|(_prefix, res)| res.split_once('@'))
            .map(|(user, domain)| (user.to_string(), domain.to_string())) else {
            return None;
        };

        if let Some((user, repository)) = user.split_once('/') {
            Some(WebFingerAcct {
                user: user.to_string(),
                domain,
                repository: Some(repository.to_string()),
            })
        } else {
            Some(WebFingerAcct {
                user,
                domain,
                repository: None,
            })
        }
    }
}

pub async fn webfinger(Query(query): Query<WebFingerQuery>) -> impl IntoResponse {
    let acct = query.parse().unwrap();

    if acct.domain == SETTINGS.domain {
        if let Some(repository) = acct.repository {
            Ok(Json(Webfinger {
                subject: query.resource,
                aliases: vec![
                    format!("http://{}/@{}/{}", acct.domain, acct.user, repository),
                    format!(
                        "http://{}/apub/users/{}/repositories/{}",
                        acct.domain, acct.user, repository
                    ),
                ],
                links: vec![Link {
                    rel: "self".to_string(),
                    href: Some(format!(
                        "http://{}/apub/users/{}/repositories/{}",
                        acct.domain, acct.user, repository
                    )),
                    template: None,
                    mime_type: Some("application/activity+json".to_string()),
                }],
            }))
        } else {
            Ok(Json(Webfinger {
                subject: query.resource,
                aliases: vec![
                    format!("http://{}/@{}", acct.domain, acct.user),
                    format!("http://{}/apub/users/{}", acct.domain, acct.user),
                ],
                links: vec![Link {
                    rel: "self".to_string(),
                    href: Some(format!("http://{}/apub/users/{}", acct.domain, acct.user)),
                    template: None,
                    mime_type: Some("application/activity+json".to_string()),
                }],
            }))
        }
    } else {
        Err(AppError::from(anyhow!("Webfinger acct not found")))
    }
}
