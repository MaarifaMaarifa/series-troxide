use std::io::ErrorKind;

use directories::ProjectDirs;
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
    pub theme: Theme,
}

const CONFIG_FILE_NAME: &str = "config.toml";

pub fn load_config() -> Config {
    if let Some(proj_dirs) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
        let mut config_file = std::path::PathBuf::from(proj_dirs.config_dir());
        config_file.push(CONFIG_FILE_NAME);

        info!("loading config file at: '{}'", config_file.display());

        let file_contents = match std::fs::read_to_string(&config_file) {
            Ok(file_contents) => file_contents,
            Err(err) => {
                let default_config = Config::default();
                if let ErrorKind::NotFound = err.kind() {
                    warn!("could not find config file at: '{}'", config_file.display());
                    if let Err(err) = std::fs::write(
                        &config_file,
                        toml::to_string_pretty(&default_config).unwrap(),
                    ) {
                        error!("Could not write default config file: {}", err);
                    }
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
                return Config::default();
            }
        }
    } else {
        error!("could not obtain config directory path");
        warn!("loading with default settings");
        Config::default()
    }
}

pub fn save_config(settings_config: &Config) {
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
