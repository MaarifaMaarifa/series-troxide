use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::RwLock;

use directories::ProjectDirs;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PATHS: RwLock<Paths> = RwLock::new(Paths::default());
}

/// Various Data paths for the program
///
/// Stores custom paths, while providing platform specific paths
/// when no custom one(s) provided
#[derive(Debug, Default, Clone)]
pub struct Paths {
    custom_data_dir_path: Option<PathBuf>,
    custom_config_dir_path: Option<PathBuf>,
    custom_cache_dir_path: Option<PathBuf>,
}

impl Paths {
    fn project_dir() -> ProjectDirs {
        ProjectDirs::from("", "", env!("CARGO_PKG_NAME")).expect("could not get the program paths")
    }

    pub fn get_data_dir_path(&self) -> Cow<PathBuf> {
        if let Some(data_path) = &self.custom_data_dir_path {
            Cow::Borrowed(data_path)
        } else {
            Cow::Owned(std::path::PathBuf::from(&Self::project_dir().data_dir()))
        }
    }

    pub fn get_config_dir_path(&self) -> Cow<PathBuf> {
        if let Some(config_path) = &self.custom_config_dir_path {
            Cow::Borrowed(config_path)
        } else {
            Cow::Owned(std::path::PathBuf::from(&Self::project_dir().config_dir()))
        }
    }

    pub fn get_cache_dir_path(&self) -> Cow<PathBuf> {
        if let Some(cache_path) = &self.custom_cache_dir_path {
            Cow::Borrowed(cache_path)
        } else {
            Cow::Owned(std::path::PathBuf::from(&Self::project_dir().cache_dir()))
        }
    }

    pub fn set_data_dir_path(&mut self, data_dir_path: PathBuf) {
        self.custom_data_dir_path = Some(data_dir_path)
    }

    pub fn set_config_dir_path(&mut self, config_dir_path: PathBuf) {
        self.custom_config_dir_path = Some(config_dir_path)
    }

    pub fn set_cache_dir_path(&mut self, cache_dir_path: PathBuf) {
        self.custom_cache_dir_path = Some(cache_dir_path)
    }
}
