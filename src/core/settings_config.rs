use serde::{Deserialize, Serialize};

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

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
}

pub fn load_config() -> Config {
    Config::default()
}

// pub fn save_config(settings_config: &Config) {}
