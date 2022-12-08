use crate::settings::Settings;
use once_cell::sync::Lazy;

pub mod api;
pub mod apub;
pub mod error;
pub mod instance;
pub mod oauth;
pub mod settings;
pub mod syntax;
pub mod view;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::get().expect("Config error"));
