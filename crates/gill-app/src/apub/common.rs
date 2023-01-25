use crate::error::{AppError, AppResult};
use crate::instance::InstanceHandle;
use activitypub_federation::core::activity_queue::send_activity;
use activitypub_federation::core::signatures::PublicKey;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use activitypub_federation::LocalInstance;
use async_session::async_trait;
use gill_db::inbox_for_url;
use gill_settings::SETTINGS;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::info;
use url::{ParseError, Url};

#[async_trait]
pub trait GillApubObject {
    fn view_uri(&self) -> String;

    async fn followers(&self, db: &InstanceHandle) -> AppResult<Vec<Url>>;

    fn local_id(&self) -> i32;

    fn public_key_with_owner(&self) -> Result<PublicKey, ParseError>;

    fn private_key(&self) -> Option<String>;

    async fn send<Activity>(
        &self,
        activity: Activity,
        recipients: Vec<Url>,
        local_instance: &LocalInstance,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Send + Sync,
        <Activity as ActivityHandler>::Error:
            From<anyhow::Error> + From<serde_json::Error> + From<AppError> + From<ParseError>,
    {
        let activity = WithContext::new_default(activity);

        info!(
            "Sending activity {} to {:?}",
            activity.id(),
            recipients.iter().map(|r| r.to_string())
        );

        send_activity(
            activity,
            self.public_key_with_owner()?,
            self.private_key().expect("has private key"),
            recipients,
            local_instance,
        )
        .await?;

        Ok(())
    }

    async fn forward_activity<Activity>(
        &self,
        activity: Activity,
        local_instance: &LocalInstance,
        db: &PgPool,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: GillActivity + ActivityHandler + Serialize + Send + Sync,
        <Activity as ActivityHandler>::Error: From<anyhow::Error>
            + From<serde_json::Error>
            + From<AppError>
            + From<ParseError>
            + From<sqlx::Error>,
    {
        let mut forward_inboxes = vec![];
        for url in activity.forward_addresses() {
            if is_local(url) {
                let inboxes = inbox_for_url(url.as_str(), db).await?;
                forward_inboxes.extend(inboxes)
            }
        }

        if !forward_inboxes.is_empty() {
            let recipients = forward_inboxes
                .into_iter()
                .filter_map(|url| Url::parse(&url).ok())
                .collect();

            info!("forwarding activity to local inboxes");
            self.send(activity, recipients, local_instance).await?
        }

        Ok(())
    }
}

pub trait GillActivity {
    fn forward_addresses(&self) -> Vec<&Url>;
}

pub fn is_local(url: &Url) -> bool {
    if SETTINGS.debug {
        let (domain, _port) = &SETTINGS.domain.split_once(':').unzip();
        &url.domain() == domain && url.port() == Some(SETTINGS.port)
    } else {
        Some(SETTINGS.domain.as_str()) == url.domain()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub content: String,
    pub media_type: String,
}
