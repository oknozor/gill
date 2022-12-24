use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateRepository {
    pub activity_pub_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub private: bool,
    pub inbox_url: String,
    pub outbox_url: String,
    pub followers_url: String,
    pub attributed_to: String,
    pub clone_uri: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub ticket_tracked_by: String,
    pub send_patches_to: String,
    pub domain: String,
    pub is_local: bool,
}
