use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub user_info_url: String,
}

impl Settings {
    pub(crate) fn get() -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(File::from(PathBuf::from("config.toml")))
            .build()?
            .try_deserialize()
    }
}
