use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::activity_queue::send_activity;
use activitypub_federation::core::signatures::PublicKey;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use activitypub_federation::LocalInstance;
use async_session::async_trait;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

#[async_trait]
pub trait GillApubObject {
    fn view_uri(&self) -> String;

    async fn followers(&self, db: &InstanceHandle) -> Result<Vec<Url>, AppError>;

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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub content: String,
    pub media_type: String,
}
