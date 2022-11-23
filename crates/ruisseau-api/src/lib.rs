use crate::settings::Settings;
use once_cell::sync::Lazy;

pub mod api;
pub mod error;
pub mod oauth;
pub mod settings;
pub mod view;
pub mod syntax;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::get().expect("Config error"));
