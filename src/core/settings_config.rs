use std::{
    io::ErrorKind,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::core::paths;

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

pub const ALL_THEMES: [Theme; 2] = [Theme::Light, Theme::Dark];

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        };

        write!(f, "{}", str)
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub appearance: AppearanceSettings,
    pub locale: LocaleSettings,
    pub notifications: NotificationSettings,
    pub custom_paths: Option<CustomPaths>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AppearanceSettings {
    pub theme: Theme,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LocaleSettings {
    pub country_code: String,
}

impl Default for LocaleSettings {
    fn default() -> Self {
        Self {
            country_code: "US".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct NotificationSettings {
    // the time is in minutes
    pub time_to_notify: u32,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self { time_to_notify: 60 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct CustomPaths {
    pub data_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
}

lazy_static! {
    pub static ref SETTINGS: Arc<RwLock<Settings>> = Arc::new(RwLock::new(Settings::new()));
}

pub struct Settings {
    current_config: Config,
    unsaved_config: Config,
}

impl Settings {
    pub fn new() -> Self {
        let config = load_config();
        Self {
            current_config: config.clone(),
            unsaved_config: config,
        }
    }

    pub fn change_settings(&mut self) -> &mut Config {
        &mut self.unsaved_config
    }

    pub fn get_current_settings(&self) -> &Config {
        &self.unsaved_config
    }

    /// Resets the settings to the initial unmodified state
    pub fn reset_settings(&mut self) {
        self.unsaved_config = self.current_config.clone();
    }

    /// Loads the default settings
    ///
    /// # Note
    /// Does not save the settings
    pub fn set_default_settings(&mut self) {
        self.unsaved_config = Config::default();
    }

    /// Checks if the unsaved settings curresponds to the
    /// default settings of the program
    pub fn has_default_settings(&self) -> bool {
        self.unsaved_config == Config::default()
    }

    pub fn has_pending_save(&self) -> bool {
        self.current_config != self.unsaved_config
    }

    pub fn save_settings(&mut self) {
        save_config(&self.unsaved_config);
        self.current_config = self.unsaved_config.clone();
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

pub const CONFIG_FILE_NAME: &str = "config.toml";

fn load_config() -> Config {
    let config_directory = paths::PATHS
        .read()
        .expect("failed to read paths")
        .get_config_dir_path()
        .to_path_buf();

    let mut config_file = config_directory.clone();
    config_file.push(CONFIG_FILE_NAME);

    info!("loading config file at: '{}'", config_file.display());

    let file_contents = match std::fs::read_to_string(&config_file) {
        Ok(file_contents) => file_contents,
        Err(err) => {
            let default_config = Config::default();
            if let ErrorKind::NotFound = err.kind() {
                warn!("could not find config file at: '{}'", config_file.display());
                std::fs::DirBuilder::new()
                    .recursive(true)
                    .create(config_directory)
                    .unwrap_or_else(|err| error!("could not create config directory: {err}"));
                std::fs::write(
                    &config_file,
                    toml::to_string_pretty(&default_config).unwrap(),
                )
                .unwrap_or_else(|err| error!("could not write default config file: {err}"));
                info!(
                    "created a new default config file at: '{}'",
                    config_file.display()
                );
            }
            return default_config;
        }
    };

    match toml::from_str(&file_contents) {
        Ok(config) => config,
        Err(err) => {
            error!("could not parse the config file: {}", err);
            warn!("loading with default settings");
            Config::default()
        }
    }
}

fn save_config(settings_config: &Config) {
    let mut config_file = paths::PATHS
        .read()
        .expect("failed to read paths")
        .get_config_dir_path()
        .to_path_buf();

    config_file.push(CONFIG_FILE_NAME);

    if let Err(err) = std::fs::write(
        &config_file,
        toml::to_string_pretty(&settings_config).unwrap(),
    ) {
        error!(
            "Could not write default config file '{}': {}",
            config_file.display(),
            err
        );
    }
}

pub mod locale_settings {
    //! Deals with interaction of GUI locale settings with the actual settings from
    //! the config file

    use super::SETTINGS;
    use rust_iso3166::ALL;

    pub fn get_country_code_from_settings() -> String {
        let country_code_str = SETTINGS
            .read()
            .unwrap()
            .get_current_settings()
            .locale
            .country_code
            .clone();

        if ALL
            .iter()
            .any(|country_code| country_code.alpha2 == country_code_str)
        {
            return country_code_str;
        }

        String::from("US")
    }

    pub fn get_country_name_from_settings() -> String {
        get_country_name_from_country_code(&get_country_code_from_settings())
            .unwrap()
            .to_owned()
    }

    pub fn get_country_code_from_country_name(country_name: &str) -> Option<&str> {
        ALL.iter()
            .find(|country_code| country_code.name == country_name)
            .map(|country_code| country_code.alpha2)
    }

    pub fn get_country_name_from_country_code(country_code_str: &str) -> Option<&str> {
        ALL.iter()
            .find(|country_code| country_code.alpha2 == country_code_str)
            .map(|country_code| country_code.name)
    }
}
