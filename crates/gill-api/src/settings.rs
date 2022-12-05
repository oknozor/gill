use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub user_info_url: String,
    pub domain: String,
    pub database_name: String,
    pub database_host: String,
    pub database_user: String,
    pub database_password: String,
}

impl Settings {
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{user}:{pwd}@{host}/{db}",
            user = self.database_user,
            pwd = self.database_password,
            host = self.database_host,
            db = self.database_name
        )
    }
}

impl Settings {
    pub(crate) fn get() -> Result<Self, config::ConfigError> {
        let config_path = PathBuf::from("config.toml");
        if !config_path.exists() {
            tracing::warn!("{config_path:?}, not found");
        }

        let mut config: Settings = Config::builder()
            .add_source(File::from(config_path))
            .build()?
            .try_deserialize()?;

        if let Ok(user_info_url) = env::var("OAUTH_USER_INFO_URL") {
            config.user_info_url = user_info_url;
        }

        if let Ok(domain) = env::var("DOMAIN") {
            config.domain = domain;
        }

        if let Ok(name) = env::var("DB_NAME") {
            config.database_name = name;
        }

        if let Ok(db_host) = env::var("DB_HOST") {
            config.database_host = db_host;
        }

        if let Ok(user) = env::var("DB_USER") {
            config.database_user = user;
        }

        if let Ok(password) = env::var("DB_PASSWORD") {
            config.database_password = password;
        }

        Ok(config)
    }
}
