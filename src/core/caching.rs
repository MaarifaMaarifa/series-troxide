use std::io::{self, ErrorKind};
use std::path;

use crate::core::api::{self, deserialize_json};

use super::api::{series_information::SeriesMainInformation, ApiError};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

const SERIES_CACHE_DIRECTORY: &str = "series-troxide-series-data";
const IMAGES_CACHE_DIRECTORY: &str = "series-troxide-images-data";
const EPISODE_LIST_FILENAME: &str = "episode-list";
const SERIES_MAIN_INFORMATION_FILENAME: &str = "main-info";

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
pub async fn load_image(image_url: String) -> Option<Vec<u8>> {
    let mut image_path = CACHER.get_cache_folder_path(CacheFolderType::Images);

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
            let images_directory = CACHER.get_cache_folder_path(CacheFolderType::Images);
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

pub async fn read_cache(cache_filepath: impl AsRef<path::Path>) -> io::Result<String> {
    fs::read_to_string(cache_filepath).await
}

pub async fn write_cache(cache_data: &str, cache_filepath: &path::Path) {
    loop {
        if let Err(err) = fs::write(cache_filepath, cache_data).await {
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

pub mod series_information {
    use super::api::series_information;
    use super::*;

    use std::io::ErrorKind;

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
        let series_information_path =
            CACHER.get_cache_file_path(CacheFilePath::SeriesMainInformation(series_id));

        let series_information_json = match read_cache(&series_information_path).await {
            Ok(json_string) => json_string,
            Err(err) => {
                info!("falling back online for 'series information' for series id: {series_id}");
                let json_string =
                    series_information::get_series_main_info_with_id(series_id).await?;

                if err.kind() == ErrorKind::NotFound {
                    write_cache(&json_string, &series_information_path).await;
                }
                json_string
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

pub mod show_images {
    use crate::core::{
        api::{
            deserialize_json,
            show_images::{get_show_images as get_show_images_api, Image, ImageType},
            ApiError,
        },
        caching::{load_image, CacheFolderType, CACHER},
    };
    use tokio::fs;
    use tracing::info;

    pub async fn get_show_images(series_id: u32) -> Result<Vec<Image>, ApiError> {
        let name = format!("{}", series_id);
        let mut series_images_list_path = CACHER.get_cache_folder_path(CacheFolderType::Series);
        series_images_list_path.push(&name); // creating the series folder path
        let series_directory = series_images_list_path.clone(); // creating a copy before we make it path to file
        series_images_list_path.push("series-images-list"); // creating the episode list json filename path

        let series_information_json = match fs::read_to_string(&series_images_list_path).await {
            Ok(info) => info,
            Err(err) => {
                info!(
                    "falling back online for 'series images list' for series id {}: {}",
                    series_id, err
                );
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(series_directory)
                    .await
                    .unwrap();
                let json_string = get_show_images_api(series_id).await?;
                fs::write(series_images_list_path, &json_string)
                    .await
                    .unwrap();
                json_string
            }
        };

        deserialize_json(&series_information_json)
    }

    /// Loads the most recent image banner from the provided series id
    pub async fn get_recent_banner(series_id: u32) -> Option<Vec<u8>> {
        let images = get_show_images(series_id).await.ok()?;
        let recent_banner = images
            .into_iter()
            .filter(|image| ImageType::new(image) != ImageType::Poster)
            .last()?;
        // .find(|image| {
        //     let image_type = ImageType::new(image);
        //     image_type == ImageType::Banner || image_type == ImageType::Background
        // })?;
        // .max_by_key(|image| image.id)?;
        println!("done getting maximum");

        load_image(recent_banner.resolutions.original.url).await

        // if (image_bytes.len() / 1_048_576) < 1 {
        //     Some(image_bytes)
        // } else {
        //     warn!("image exceeded 1MB for series id {series_id}");
        //     None
        // }
    }
}

pub mod episode_list {
    use std::{collections::HashSet, io::ErrorKind};

    use crate::core::{
        api::{
            deserialize_json,
            episodes_information::{get_episode_list, Episode},
            ApiError,
        },
        caching::CACHER,
    };
    use chrono::{DateTime, Datelike, Local, Timelike, Utc};
    use tracing::info;

    use super::{read_cache, write_cache, CacheFilePath};

    #[derive(Clone, Debug)]
    pub struct EpisodeList {
        episodes: Vec<Episode>,
    }

    impl EpisodeList {
        pub async fn new(series_id: u32) -> Result<Self, ApiError> {
            let episodes_list_path =
                CACHER.get_cache_file_path(CacheFilePath::SeriesEpisodeList(series_id));

            let json_string = match read_cache(&episodes_list_path).await {
                Ok(json_string) => json_string,
                Err(err) => {
                    info!("falling back online for 'episode_list' for series id: {series_id}");
                    let (episodes, json_string) = get_episode_list(series_id).await?;

                    if err.kind() == ErrorKind::NotFound {
                        write_cache(&json_string, &episodes_list_path).await;
                    }
                    return Ok(Self { episodes });
                }
            };

            let episodes = deserialize_json::<Vec<Episode>>(&json_string)?;
            Ok(Self { episodes })
        }

        pub fn get_episode(&self, season_number: u32, episode_number: u32) -> Option<&Episode> {
            self.episodes.iter().find(|episode| {
                (episode.season == season_number) && (episode.number == Some(episode_number))
            })
        }

        pub fn get_episodes(&self, season: u32) -> Vec<&Episode> {
            self.episodes
                .iter()
                .filter(|episode| episode.season == season)
                .collect()
        }

        // /// Get the total number of all episodes in the Series
        // pub fn get_total_episodes(&self) -> usize {
        //     self.episodes.len()
        // }

        /// Get the total number of all watchable episodes in the Series
        pub fn get_total_watchable_episodes(&self) -> usize {
            self.episodes
                .iter()
                .filter(|episode| Self::is_episode_watchable(episode) == Some(true))
                .count()
        }

        /// Returns the number of all seasons available and their total episodes as a tuple (season_no, total_episodes)
        pub fn get_season_numbers_with_total_episode(&self) -> Vec<(u32, TotalEpisodes)> {
            let seasons: HashSet<u32> =
                self.episodes.iter().map(|episode| episode.season).collect();
            let mut seasons: Vec<u32> = seasons.iter().copied().collect();
            seasons.sort();

            seasons
                .into_iter()
                .map(|season| {
                    let total_episodes = self.get_episodes(season).into_iter().count();
                    let total_watchable_episodes = self
                        .get_episodes(season)
                        .into_iter()
                        .filter(|episode| Self::is_episode_watchable(episode) == Some(true))
                        .count();
                    (
                        season,
                        TotalEpisodes::new(total_episodes, total_watchable_episodes),
                    )
                })
                .collect()
        }

        /// Returns the number of all seasons available and their total episodes as a tuple (season_no, total_episodes)
        pub fn get_season_numbers_with_total_watchable_episode(&self) -> Vec<(u32, usize)> {
            let seasons: HashSet<u32> =
                self.episodes.iter().map(|episode| episode.season).collect();
            let mut seasons: Vec<u32> = seasons.iter().copied().collect();
            seasons.sort();

            seasons
                .into_iter()
                .map(|season| {
                    let total_episodes = self
                        .get_episodes(season)
                        .into_iter()
                        .filter(|episode| Self::is_episode_watchable(episode) == Some(true))
                        .count();
                    (season, total_episodes)
                })
                .collect()
        }

        /// Tells if the episode is watchable or not based on the current time and the episode release time
        ///
        /// This method returns an optional bool as an episode my not have airstamp associated with it hence
        /// the method can not infer that information.
        pub fn is_episode_watchable(episode: &Episode) -> Option<bool> {
            let airstamp = DateTime::parse_from_rfc3339(episode.airstamp.as_ref()?)
                .unwrap()
                .with_timezone(&Local);
            let local_time = Utc::now().with_timezone(&Local);
            Some(airstamp <= local_time)
        }

        /// Returns the previous episode from the current time
        ///
        /// This method is also useful when finding the maximum watchable episode
        /// as you can not watch an episode that is released in the future.
        pub fn get_previous_episode(&self) -> Option<&Episode> {
            let mut episodes_iter = self.episodes.iter().peekable();
            while let Some(episode) = episodes_iter.next() {
                if let Some(peeked_episode) = episodes_iter.peek() {
                    if !Self::is_episode_watchable(&peeked_episode)? {
                        return Some(episode);
                    }
                } else {
                    return Some(episode);
                }
            }
            None
        }

        /// Returns the next episode from the current time
        pub fn get_next_episode(&self) -> Option<&Episode> {
            self.episodes
                .iter()
                .find(|episode| Self::is_episode_watchable(episode) == Some(false))
        }

        /// Returns the next episode and it's release time
        pub fn get_next_episode_and_time(&self) -> Option<(&Episode, EpisodeReleaseTime)> {
            let next_episode = self.get_next_episode()?;
            let next_episode_airstamp = next_episode.airstamp.as_ref()?;

            let release_time = EpisodeReleaseTime::from_rfc3339_str(next_episode_airstamp);
            Some((next_episode, release_time))
        }
    }

    #[derive(Clone, Debug)]
    pub struct TotalEpisodes {
        all_episodes: usize,
        all_watchable_episodes: usize,
    }

    impl TotalEpisodes {
        fn new(all_episodes: usize, all_watchable_episodes: usize) -> Self {
            Self {
                all_episodes,
                all_watchable_episodes,
            }
        }

        /// Retrieves all the episodes
        pub fn get_all_episodes(&self) -> usize {
            self.all_episodes
        }

        /// Retrieves all the watchable episodes
        pub fn get_all_watchable_episodes(&self) -> usize {
            self.all_watchable_episodes
        }
    }

    #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
    pub struct EpisodeReleaseTime {
        release_time: DateTime<Local>,
    }

    impl EpisodeReleaseTime {
        pub fn new(release_time: DateTime<Utc>) -> Self {
            Self {
                release_time: release_time.with_timezone(&Local),
            }
        }

        fn from_rfc3339_str(str: &str) -> Self {
            Self {
                release_time: DateTime::parse_from_rfc3339(str)
                    .unwrap()
                    .with_timezone(&Local),
            }
        }

        /// Returns the remaining time for an episode to be released
        pub fn get_remaining_release_time(&self) -> Option<String> {
            let local_time = Utc::now().with_timezone(&Local);

            if self.release_time > local_time {
                let time_diff = self.release_time - local_time;

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

        /// Returns the remaining full date and time for an episode to be released
        pub fn get_full_release_date_and_time(&self) -> String {
            /// appends zero the minute digit if it's below 10 for better display
            fn append_zero(num: u32) -> String {
                if num < 10 {
                    format!("0{num}")
                } else {
                    format!("{num}")
                }
            }

            let (is_pm, hour) = self.release_time.hour12();
            let pm_am = if is_pm { "p.m." } else { "a.m." };

            let minute = append_zero(self.release_time.minute());

            format!(
                "{} {} {}:{} {}",
                self.release_time.date_naive(),
                self.release_time.weekday(),
                hour,
                minute,
                pm_am
            )
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
