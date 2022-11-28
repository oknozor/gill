use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub user_info_url: String,
    pub repo_dir: PathBuf,
    pub domain: String,
}

impl Settings {
    pub(crate) fn get() -> Result<Self, config::ConfigError> {
        let config_path = PathBuf::from("config.toml");
        if !config_path.exists() {
            tracing::warn!("{config_path:?}, not found");
        }

        Config::builder()
            .add_source(File::from(config_path))
            .build()?
            .try_deserialize()
    }
}
