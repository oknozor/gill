use serde_json::Value;
use serde::Deserialize;
use crate::SETTINGS;

#[derive(Debug, Deserialize)]
pub struct Oauth2Token {
    pub(crate) claims: Value,
}

/// A trait to access JWT claims
pub trait Oauth2UserInfo {
    /// Return the role associated with this user
    fn roles(&self) -> Vec<&str>;

    /// Return the name of the user
    fn name(&self) -> Option<&str>;
}

impl Oauth2UserInfo for Oauth2Token {
    fn roles(&self) -> Vec<&str> {
        self.claims
            .as_object()
            .and_then(|claims| claims.get("realm_access"))
            .and_then(|claims| claims.get("roles"))
            .and_then(|roles| roles.as_array())
            .and_then(|roles| roles.iter().map(|role| role.as_str()).collect())
            .unwrap_or_default()
    }

    fn name(&self) -> Option<&str> {
        self.claims
            .as_object()
            .and_then(|claims| claims.get("name"))
            .and_then(|name| name.as_str())
    }
}

pub async fn user_info(client: reqwest::Client, bearer: &str) -> reqwest::Result<Oauth2Token> {
    client
        .get(&SETTINGS.user_info_url)
        .header("Authorization", bearer)
        .send()
        .await?
        .json()
        .await

}