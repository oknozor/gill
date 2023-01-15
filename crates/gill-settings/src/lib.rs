use config::{Config, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::get().expect("Config error"));

const OAUTH_CLIENT_ID: &str = "GILL_OAUTH_CLIENT_ID";
const OAUTH_CLIENT_SECRET: &str = "GILL_OAUTH_CLIENT_SECRET";
const OAUTH_PROVIDER: &str = "GILL_OAUTH_PROVIDER";
const OAUTH_USER_INFO_URL: &str = "GILL_OAUTH_USER_INFO_URL";
const OAUTH_TOKEN_URL: &str = "GILL_OAUTH_TOKEN_URL";
const OAUTH_AUTH_URL: &str = "GILL_OAUTH_AUTH_URL";
const DOMAIN: &str = "GILL_DOMAIN";
const DB_NAME: &str = "GILL_DB_NAME";
const DB_HOST: &str = "GILL_DB_HOST";
const DB_USER: &str = "GILL_DB_USER";
const DB_PORT: &str = "GILL_DB_PORT";
const DB_PASSWORD: &str = "GILL_DB_PASSWORD";
const PORT: &str = "GILL_PORT";
const DEBUG: &str = "GILL_DEBUG";

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub domain: String,
    pub debug: bool,
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
    pub fn protocol(&self) -> &str {
        if self.debug {
            "http"
        } else {
            "https"
        }
    }

    pub(crate) fn get() -> Result<Self, config::ConfigError> {
        let config_path = PathBuf::from("/home/git/config.toml");

        if config_path.exists() {
            let config: Settings = Config::builder()
                .add_source(File::from(config_path))
                .build()?
                .try_deserialize()?;

            Ok(config.override_with_env())
        } else {
            tracing::warn!("{config_path:?}, not found");
            Ok(Settings::from_env())
        }
    }

    fn override_with_env(mut self) -> Self {
        if let Ok(client_id) = env::var(OAUTH_CLIENT_ID) {
            self.oauth_provider.client_id = client_id;
        }

        if let Ok(client_secret) = env::var(OAUTH_CLIENT_SECRET) {
            self.oauth_provider.client_secret = client_secret;
        }

        if let Ok(provider) = env::var(OAUTH_PROVIDER) {
            self.oauth_provider.provider = provider;
        }

        if let Ok(user_info_url) = env::var(OAUTH_USER_INFO_URL) {
            self.oauth_provider.user_info_url = user_info_url;
        }

        if let Ok(token_url) = env::var(OAUTH_TOKEN_URL) {
            self.oauth_provider.token_url = token_url;
        }

        if let Ok(auth_url) = env::var(OAUTH_AUTH_URL) {
            self.oauth_provider.auth_url = auth_url;
        }

        if let Ok(domain) = env::var(DOMAIN) {
            self.domain = domain;
        }

        if let Ok(name) = env::var(DB_NAME) {
            self.database.database = name;
        }

        if let Ok(db_host) = env::var(DB_HOST) {
            self.database.host = db_host;
        }

        if let Ok(user) = env::var(DB_USER) {
            self.database.user = user;
        }

        if let Ok(port) = env::var(DB_PORT) {
            self.database.port = port.parse().expect("Invalid port number");
        }

        if let Ok(password) = env::var(DB_PASSWORD) {
            self.database.password = password;
        }

        if let Ok(port) = env::var(PORT) {
            self.port = port.parse().expect("Invalid port number");
        }

        if let Ok(debug) = env::var(DEBUG) {
            self.debug = debug.parse().expect("Expected a bool");
        }

        self
    }

    fn from_env() -> Self {
        Settings {
            domain: env::var(DOMAIN).expect("Missing env var 'DOMAIN'"),
            debug: env::var(DEBUG)
                .expect(DEBUG)
                .parse()
                .expect("GILL_DEBUG must be a bool"),
            port: env::var(PORT)
                .expect(PORT)
                .parse()
                .expect("GILL_PORT must be an integer"),
            oauth_provider: AuthSettings {
                client_id: env::var(OAUTH_CLIENT_ID).expect(OAUTH_CLIENT_ID),
                client_secret: env::var(OAUTH_CLIENT_SECRET).expect(OAUTH_CLIENT_SECRET),
                provider: env::var(OAUTH_PROVIDER).expect(OAUTH_PROVIDER),
                user_info_url: env::var(OAUTH_USER_INFO_URL).expect(OAUTH_USER_INFO_URL),
                auth_url: env::var(OAUTH_AUTH_URL).expect(OAUTH_AUTH_URL),
                token_url: env::var(OAUTH_TOKEN_URL).expect(OAUTH_TOKEN_URL),
            },
            database: DbSettings {
                database: env::var(DB_NAME).expect(DB_NAME),
                host: env::var(DB_HOST).expect(DB_HOST),
                port: env::var(DB_PORT)
                    .expect(DB_PORT)
                    .parse()
                    .expect("GILL_DB_PORT must be an integer"),
                user: env::var(DB_USER).expect(DB_USER),
                password: env::var(DB_PASSWORD).expect(DB_PASSWORD),
            },
        }
    }
}
