use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use once_cell::sync::Lazy;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::get().expect("Config error"));

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub domain: String,
    pub port: u16,
    pub oauth_provider: AuthSettings,
    pub database: DbSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbSettings {
    pub database: String,
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSettings {
    client_id: String,
    client_secret: String,
    provider: String,
    user_info_url: String,
    auth_url: String,
    token_url: String,
}

impl AuthSettings {
    pub fn client_id(&self) -> String {
        self.client_id.to_string()
    }

    pub fn client_secret(&self) -> String {
        self.client_secret.to_string()
    }

    pub fn token_url(&self) -> String {
        format!("{}{}", self.provider, self.token_url)
    }

    pub fn user_info_url(&self) -> String {
        format!("{}{}", self.provider, self.user_info_url)
    }

    pub fn auth_url(&self) -> String {
        format!("{}{}", self.provider, self.auth_url)
    }
}

impl Settings {
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{user}:{pwd}@{host}:{port}/{db}",
            user = self.database.user,
            pwd = self.database.password,
            host = self.database.host,
            port = self.database.port,
            db = self.database.database
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

        if let Ok(client_id) = env::var("OAUTH_CLIENT_ID") {
            config.oauth_provider.client_id = client_id;
        }

        if let Ok(client_secret) = env::var("OAUTH_CLIENT_SECRET") {
            config.oauth_provider.client_secret = client_secret;
        }

        if let Ok(provider) = env::var("OAUTH_PROVIDER") {
            config.oauth_provider.provider = provider;
        }

        if let Ok(user_info_url) = env::var("OAUTH_USER_INFO_URL") {
            config.oauth_provider.user_info_url = user_info_url;
        }

        if let Ok(token_url) = env::var("OAUTH_TOKEN_URL") {
            config.oauth_provider.token_url = token_url;
        }

        if let Ok(auth_url) = env::var("OAUTH_AUTH_URL") {
            config.oauth_provider.auth_url = auth_url;
        }

        if let Ok(domain) = env::var("DOMAIN") {
            config.domain = domain;
        }

        if let Ok(name) = env::var("DB_NAME") {
            config.database.database = name;
        }

        if let Ok(db_host) = env::var("DB_HOST") {
            config.database.host = db_host;
        }

        if let Ok(user) = env::var("DB_USER") {
            config.database.user = user;
        }

        if let Ok(port) = env::var("DB_PORT") {
            config.database.port = port.parse()
                .expect("Invalid port number");
        }

        if let Ok(password) = env::var("DB_PASSWORD") {
            config.database.password = password;
        }

        Ok(config)
    }
}