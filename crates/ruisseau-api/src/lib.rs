use crate::settings::Settings;
use once_cell::sync::Lazy;

pub mod db;
pub mod error;
pub mod oauth;
pub mod route;
pub mod settings;

pub const SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::get().expect("Config error"));
