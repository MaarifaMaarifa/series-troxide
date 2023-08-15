//! Caching for Series and Images data
//!
//! Due to the rate limit of the api and avoiding overwhelming the api i.e. when
//! computing statistics for all the series in the database, caching becomes one
//! of the most important aspects of series troxide.
//!
//! # Caching Strategy
//! The current implementation of caching for series troxide involves two directories
//! series cache directory and images cache directory.
//!
//! ## Series cache directory
//! Series cache directory stores folders for individual series named after their series
//! ids corresponding to their ids provided by the tvmaze api. Each series folder contains
//! four files which are all in json format.
//! - `main-info`. The main series information.
//! - `episode-list`. The list of all episode of the series.
//! - `show-cast`. The list of top cast of the series.
//! - `image-list`. The list of all images of the series i.e posters, banners, backgrounds etc.
//!
//! ## Images cache directory
//! This contains all the images ever loaded by series troxide. Having all images ever loaded by
//! the program in the same directory makes it more efficient by reusing the same images that might
//! be required in different places. A good example is the show cast, the same actor can appear
//! in different shows and tvmaze reuses the same image for that actor. Another example is when
//! searching for series, results may include a series that is already tracked and whose image
//! is already cached.

use bytes::Bytes;
use std::io::{self, ErrorKind};
use std::path;

use crate::core::api::{self, deserialize_json};

use super::api::{series_information::SeriesMainInformation, ApiError};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use tokio::fs;
use tracing::{error, info};

pub mod cache_cleaning;
pub mod episode_list;
pub mod series_information;
pub mod series_list;
pub mod show_cast;
pub mod show_images;
pub mod tv_schedule;

const SERIES_CACHE_DIRECTORY: &str = "series-cache";
const IMAGES_CACHE_DIRECTORY: &str = "images-cache";
const EPISODE_LIST_FILENAME: &str = "episode-list";
const SERIES_MAIN_INFORMATION_FILENAME: &str = "main-info";
const SERIES_CAST_FILENAME: &str = "show-cast";
const SERIES_IMAGE_LIST_FILENAME: &str = "image-list";

lazy_static! {
    pub static ref CACHER: Cacher = Cacher::init();
}

pub enum CacheFolderType {
    Series,
    Images,
}

pub enum CacheFilePath {
    SeriesMainInformation(u32),
    SeriesEpisodeList(u32),
    SeriesShowCast(u32),
    SeriesImageList(u32),
}

pub struct Cacher {
    cache_path: path::PathBuf,
}

impl Cacher {
    pub fn init() -> Self {
        info!("opening cache");

        let proj_dir = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
            .expect("could not get the cache path");

        let cache_path = path::PathBuf::from(&proj_dir.cache_dir());
        Self { cache_path }
    }

    /// Return the root path where all series troxide data resides including
    /// the cache
    pub fn get_project_path(&self) -> &path::Path {
        &self.cache_path
    }

    pub fn get_cache_folder_path(&self, cache_type: CacheFolderType) -> path::PathBuf {
        let mut cache_path = self.cache_path.clone();
        match cache_type {
            CacheFolderType::Series => cache_path.push(SERIES_CACHE_DIRECTORY),
            CacheFolderType::Images => cache_path.push(IMAGES_CACHE_DIRECTORY),
        }
        cache_path
    }

    /// This method is used to retrieve cache files for individual files in the series cache directory
    /// i.e episode-list, main-info
    pub fn get_cache_file_path(&self, cache_file_type: CacheFilePath) -> path::PathBuf {
        match cache_file_type {
            CacheFilePath::SeriesMainInformation(series_id) => {
                let mut cache_folder = self.get_series_cache_folder_path(series_id);
                cache_folder.push(SERIES_MAIN_INFORMATION_FILENAME);
                cache_folder
            }
            CacheFilePath::SeriesEpisodeList(series_id) => {
                let mut cache_folder = self.get_series_cache_folder_path(series_id);
                cache_folder.push(EPISODE_LIST_FILENAME);
                cache_folder
            }
            CacheFilePath::SeriesShowCast(series_id) => {
                let mut cache_folder = self.get_series_cache_folder_path(series_id);
                cache_folder.push(SERIES_CAST_FILENAME);
                cache_folder
            }
            CacheFilePath::SeriesImageList(series_id) => {
                let mut cache_folder = self.get_series_cache_folder_path(series_id);
                cache_folder.push(SERIES_IMAGE_LIST_FILENAME);
                cache_folder
            }
        }
    }

    /// This method is used to retrieve the series folder path that is a parent to individual cache files
    pub fn get_series_cache_folder_path(&self, series_id: u32) -> path::PathBuf {
        let mut cache_folder = self.get_cache_folder_path(CacheFolderType::Series);
        cache_folder.push(format!("{series_id}"));
        cache_folder
    }
}

/// Loads the image from the provided url
pub async fn load_image(image_url: String) -> Option<Bytes> {
    // Hashing the image url as a file name as the forward slashes in web urls
    // mimic paths
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(&image_url);
    let image_hash = format!("{:x}", hasher.finalize());

    let mut image_path = CACHER.get_cache_folder_path(CacheFolderType::Images);
    image_path.push(&image_hash);

    match fs::read(&image_path).await {
        Ok(image_bytes) => Some(Bytes::from(image_bytes)),
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                info!("falling back online for image with link {}", image_url);
                if let Some(image_bytes) = api::lload_image(image_url).await {
                    write_cache(&image_bytes, &image_path).await;
                    Some(image_bytes)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

pub async fn read_cache(cache_filepath: impl AsRef<path::Path>) -> io::Result<String> {
    fs::read_to_string(cache_filepath).await
}

pub async fn write_cache(cache_data: impl AsRef<[u8]>, cache_filepath: &path::Path) {
    loop {
        if let Err(err) = fs::write(cache_filepath, &cache_data).await {
            if err.kind() == ErrorKind::NotFound {
                let mut cache_folder = path::PathBuf::from(cache_filepath);
                cache_folder.pop();
                match fs::create_dir_all(&cache_folder).await {
                    Err(err) => {
                        error!(
                            "failed to create cache directory '{}': {}",
                            cache_folder.display(),
                            err
                        );
                        break;
                    }
                    Ok(_) => continue,
                };
            } else {
                error!(
                    "failed to write cache '{}': {}",
                    cache_filepath.display(),
                    err
                );
                break;
            }
        }
        break;
    }
}
