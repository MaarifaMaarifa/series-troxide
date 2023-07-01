use std::path;

use crate::core::api::{self, deserialize_json};

use super::api::{series_information::SeriesMainInformation, ApiError};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::info;

const SERIES_CACHE_DIRECTORY: &str = "series-troxide-series-data";
const IMAGES_CACHE_DIRECTORY: &str = "series-troxide-images-data";

lazy_static! {
    pub static ref CACHER: Cacher = Cacher::init();
}

pub enum CacheType {
    Series,
    Images,
}

pub struct Cacher {
    cache_path: path::PathBuf,
}

impl Cacher {
    pub fn init() -> Self {
        info!("opening cache");
        if let Some(proj_dir) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
            let cache_path = path::PathBuf::from(&proj_dir.data_dir());

            return Self { cache_path };
        } else {
            panic!("could not get the cache path");
        }
    }

    pub fn get_cache_path(&self, cache_type: CacheType) -> path::PathBuf {
        let mut cache_path = self.cache_path.clone();
        match cache_type {
            CacheType::Series => cache_path.push(SERIES_CACHE_DIRECTORY),
            CacheType::Images => cache_path.push(IMAGES_CACHE_DIRECTORY),
        }
        cache_path
    }
}

/// Loads the image from the provided url
pub async fn load_image(image_url: String) -> Option<Vec<u8>> {
    let mut image_path = CACHER.get_cache_path(CacheType::Images);

    // Hashing the image url as a file name as the forward slashes in web urls
    // mimic paths
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(&image_url);
    let image_hash = format!("{:x}", hasher.finalize());

    image_path.push(&image_hash);

    match fs::read(&image_path).await {
        Ok(image_bytes) => return Some(image_bytes),
        Err(err) => {
            let images_directory = CACHER.get_cache_path(CacheType::Images);
            if !images_directory.exists() {
                info!("creating images cache directory as none exists");
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(images_directory)
                    .await
                    .unwrap();
            };
            info!(
                "falling back online for image with link {}: {}",
                image_url, err
            );
            return if let Some(image_bytes) = api::lload_image(image_url).await {
                let mut image_file = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(image_path)
                    .await
                    .unwrap();
                image_file.write(&image_bytes).await.unwrap();
                Some(image_bytes)
            } else {
                None
            };
        }
    };
}

pub mod series_information {
    use super::api::series_information;
    use super::*;

    pub async fn get_series_main_info_with_url(
        url: String,
    ) -> Result<SeriesMainInformation, ApiError> {
        let id = url
            .split('/')
            .last()
            .expect("invalid url, no series id at the end of url")
            .parse::<u32>()
            .expect("could not parse series id from url");

        get_series_main_info_with_id(id).await
    }

    pub async fn get_series_main_info_with_id(
        series_id: u32,
    ) -> Result<SeriesMainInformation, ApiError> {
        let name = format!("{}", series_id);
        let mut series_main_info_path = CACHER.get_cache_path(CacheType::Series);
        series_main_info_path.push(&name); // creating the series folder path
        let series_directory = series_main_info_path.clone(); // creating a copy before we make it path to file
        series_main_info_path.push(&name); // creating the series information json filename path

        let series_information_json = match fs::read_to_string(&series_main_info_path).await {
            Ok(info) => info,
            Err(err) => {
                info!(
                    "falling back online for 'series information' with id {}: {}",
                    series_id, err
                );
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(series_directory)
                    .await
                    .unwrap();
                let (series_information, json_string) =
                    series_information::get_series_main_info_with_id(series_id)
                        .await?
                        .get_data();
                fs::write(series_main_info_path, json_string).await.unwrap();
                return Ok(series_information);
            }
        };

        deserialize_json(&series_information_json)
    }

    pub async fn get_series_main_info_with_ids(
        series_ids: Vec<String>,
    ) -> Vec<SeriesMainInformation> {
        let handles: Vec<_> = series_ids
            .iter()
            .map(|id| tokio::spawn(get_series_main_info_with_id(id.parse().unwrap())))
            .collect();

        let mut series_infos = Vec::with_capacity(handles.len());
        for handle in handles {
            let series_info = handle.await.unwrap().unwrap();
            series_infos.push(series_info);
        }
        series_infos
    }
}

pub mod episode_list {
    use std::collections::HashSet;

    use crate::core::{
        api::{
            deserialize_json,
            episodes_information::{get_episode_list, Episode},
            ApiError,
        },
        caching::{CacheType, CACHER},
    };
    use chrono::{DateTime, Local, Utc};
    use tokio::fs;
    use tracing::info;

    pub struct EpisodeList {
        episodes: Vec<Episode>,
    }

    impl EpisodeList {
        pub async fn new(series_id: u32) -> Result<Self, ApiError> {
            let name = format!("{}", series_id);
            let mut episode_list_path = CACHER.get_cache_path(CacheType::Series);
            episode_list_path.push(&name); // creating the series folder path
            let series_directory = episode_list_path.clone(); // creating a copy before we make it path to file
            episode_list_path.push("episode-list"); // creating the episode list json filename path

            let series_information_json = match fs::read_to_string(&episode_list_path).await {
                Ok(info) => info,
                Err(err) => {
                    info!(
                        "falling back online for 'episode list' for series id {}: {}",
                        series_id, err
                    );
                    fs::DirBuilder::new()
                        .recursive(true)
                        .create(series_directory)
                        .await
                        .unwrap();
                    let (episodes, json_string) = get_episode_list(series_id).await?;
                    fs::write(episode_list_path, json_string).await.unwrap();
                    return Ok(Self { episodes });
                }
            };

            let episodes = deserialize_json::<Vec<Episode>>(&series_information_json)?;
            Ok(Self { episodes })
        }

        pub fn get_episodes(&self, season: u32) -> Vec<&Episode> {
            self.episodes
                .iter()
                .filter(|episode| episode.season == season)
                .collect()
        }

        /// Get the total number of all episodes in the Series
        pub fn get_total_episodes(&self) -> usize {
            self.episodes.len()
        }

        /// Returns the number of all seasons available and their total episodes as a tuple (season_no, total_episodes)
        pub fn get_season_numbers_with_total_episode(&self) -> Vec<(u32, usize)> {
            let seasons: HashSet<u32> =
                self.episodes.iter().map(|episode| episode.season).collect();
            let mut seasons: Vec<u32> = seasons.iter().copied().collect();
            seasons.sort();

            seasons
                .into_iter()
                .map(|season| {
                    let total_episodes = self.get_episodes(season).into_iter().count();
                    (season, total_episodes)
                })
                .collect()
        }

        /// Returns the next episode from the current time
        pub fn get_next_episode(&self) -> Option<&Episode> {
            self.episodes.iter().find(|episode| {
                if let Some(airstamp) = episode.airstamp.as_ref() {
                    let airstamp = DateTime::parse_from_rfc3339(&airstamp)
                        .unwrap()
                        .with_timezone(&Local);
                    let local_time = Utc::now().with_timezone(&Local);
                    airstamp > local_time
                } else {
                    false
                }
            })
        }
    }

    /// Returns the remaining time for an episode to be released
    pub fn get_release_remaining_time(episode: &Episode) -> Option<String> {
        let airstamp = DateTime::parse_from_rfc3339(episode.airstamp.as_ref()?)
            .unwrap()
            .with_timezone(&Local);
        let local_time = Utc::now().with_timezone(&Local);

        if airstamp > local_time {
            let time_diff = airstamp - local_time;

            if time_diff.num_weeks() != 0 {
                return Some(format!("{} weeks", time_diff.num_weeks()));
            }
            if time_diff.num_days() != 0 {
                return Some(format!("{} days", time_diff.num_days()));
            }
            if time_diff.num_hours() != 0 {
                return Some(format!("{} hours", time_diff.num_hours()));
            }
            if time_diff.num_minutes() != 0 {
                return Some(format!("{} minutes", time_diff.num_minutes()));
            }
            Some(String::from("Now"))
        } else {
            None
        }
    }
}
