use serde::Deserialize;

#[cfg(not(feature = "integration"))]
pub mod service;

#[cfg(feature = "integration")]
pub mod service_mock;
#[cfg(feature = "integration")]
pub use service_mock as service;

#[derive(Debug, Clone, Deserialize)]
pub struct Oauth2User {
    pub sub: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}
