use std::{io::ErrorKind, sync::RwLock};

use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

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
    pub cache: CacheSettings,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AppearanceSettings {
    pub theme: Theme,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CacheSettings {
    // the frequencies are in days
    pub aired_cache_clean_frequency: u32,
    pub ended_cache_clean_frequency: u32,
    pub waiting_release_cache_clean_frequency: u32,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            aired_cache_clean_frequency: 1,
            ended_cache_clean_frequency: 7,
            waiting_release_cache_clean_frequency: 2,
        }
    }
}

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new());
}

pub struct Settings {
    current_config: Config,
    unsaved_config: Config,
}

impl Settings {
    fn new() -> Self {
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

const CONFIG_FILE_NAME: &str = "config.toml";

fn load_config() -> Config {
    if let Some(proj_dirs) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
        let config_directory = std::path::PathBuf::from(proj_dirs.config_dir());
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
    } else {
        error!("could not obtain config directory path");
        warn!("loading with default settings");
        Config::default()
    }
}

fn save_config(settings_config: &Config) {
    if let Some(proj_dirs) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
        let mut config_file = std::path::PathBuf::from(proj_dirs.config_dir());
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
    } else {
        error!("could not obtain config directory path when saving the settings");
    }
}
