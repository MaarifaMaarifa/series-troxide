//! # Cache cleaning implementations
//!
//! Since the program keeps cache to avoid performing too many requests,
//! we need some form of cache cleaning so that we don't compromise the
//! ability of getting up to date information.

use std::path;

use crate::core::{
    api::{deserialize_json, series_information::SeriesMainInformation},
    settings_config::CacheSettings,
};

use super::{
    episode_list::EpisodeList, read_cache, CacheFolderType, CACHER, EPISODE_LIST_FILENAME,
    SERIES_MAIN_INFORMATION_FILENAME,
};
use anyhow::bail;
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{error, info};

const CLEANING_RECORD_FILENAME: &str = "cache-cleaning-record.toml";

/// A type of cleaning to be performed by cache cleaner
pub enum CleanType {
    /// for series that are running.
    Running(RunningStatus),
    /// for series that have ended.
    Ended,
}

/// Running status of a series
pub enum RunningStatus {
    /// for series that are actively being aired.
    Aired,
    /// for series that are not actively being aired but still
    /// running.
    WaitingRelease,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheCleaningRecord {
    pub last_aired_cache_clean: DateTime<Local>,
    pub last_waiting_release_cache_clean: DateTime<Local>,
    pub last_ended_cache_clean: DateTime<Local>,
}

pub struct CacheCleaner {
    cache_cleaning_record: CacheCleaningRecord,
}

impl CacheCleaner {
    fn get_cache_cleaning_record_path() -> path::PathBuf {
        let mut cache_path = path::PathBuf::from(CACHER.get_project_path());
        cache_path.push(CLEANING_RECORD_FILENAME);
        cache_path
    }

    fn open_cleaning_record() -> anyhow::Result<CacheCleaningRecord> {
        let cache_cleaning_record_path = Self::get_cache_cleaning_record_path();

        let content = match std::fs::read_to_string(&cache_cleaning_record_path) {
            Ok(content) => content,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    let cache_cleaning_record = CacheCleaningRecord::default();
                    let content = toml::to_string(&cache_cleaning_record)?;
                    std::fs::write(cache_cleaning_record_path, content)?;
                    return Ok(cache_cleaning_record);
                } else {
                    bail!(err)
                }
            }
        };
        Ok(toml::from_str(&content)?)
    }

    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            cache_cleaning_record: Self::open_cleaning_record()?,
        })
    }

    pub async fn clean_cache(&mut self, clean_type: CleanType) -> anyhow::Result<()> {
        match clean_type {
            CleanType::Running(running_status) => {
                clean_running_cache(&running_status).await?;
                match running_status {
                    RunningStatus::Aired => {
                        self.cache_cleaning_record.last_aired_cache_clean =
                            Utc::now().with_timezone(&Local)
                    }

                    RunningStatus::WaitingRelease => {
                        self.cache_cleaning_record.last_waiting_release_cache_clean =
                            Utc::now().with_timezone(&Local)
                    }
                }
            }
            CleanType::Ended => {
                clean_ended_series_cache().await?;
                self.cache_cleaning_record.last_ended_cache_clean =
                    Utc::now().with_timezone(&Local);
            }
        }

        std::fs::write(
            Self::get_cache_cleaning_record_path(),
            toml::to_string(&self.cache_cleaning_record)?,
        )?;

        Ok(())
    }

    pub async fn auto_clean(&mut self, cache_settings: &CacheSettings) -> anyhow::Result<()> {
        info!("running cache autoclean");

        let local_time = Utc::now().with_timezone(&Local);

        // Getting how many days have lasted since each type of clean was performed
        let last_aired_clean_days = local_time - self.cache_cleaning_record.last_aired_cache_clean;
        let last_ended_clean_days = local_time - self.cache_cleaning_record.last_ended_cache_clean;
        let last_waiting_release_clean_days =
            local_time - self.cache_cleaning_record.last_waiting_release_cache_clean;

        // Checking if those lasted days exceeded the required time as set by CacheSetting and perform a clean
        if last_aired_clean_days > Duration::days(cache_settings.aired_cache_clean_frequency as i64)
        {
            info!("cleaning 'airing series cache'");
            self.clean_cache(CleanType::Running(RunningStatus::Aired))
                .await?;
        }

        if last_ended_clean_days > Duration::days(cache_settings.ended_cache_clean_frequency as i64)
        {
            info!("cleaning 'ended series cache'");
            self.clean_cache(CleanType::Ended).await?;
        }

        if last_waiting_release_clean_days
            > Duration::days(cache_settings.waiting_release_cache_clean_frequency as i64)
        {
            info!("cleaning 'waiting release series cache'");
            self.clean_cache(CleanType::Running(RunningStatus::WaitingRelease))
                .await?;
        }

        Ok(())
    }
}

/// Cleans the cache of all ended series
async fn clean_ended_series_cache() -> anyhow::Result<()> {
    let mut read_dir = fs::read_dir(CACHER.get_cache_folder_path(CacheFolderType::Series)).await?;

    while let Some(dir_entry) = read_dir.next_entry().await? {
        let mut series_main_info_path = dir_entry.path();
        series_main_info_path.push(SERIES_MAIN_INFORMATION_FILENAME);

        let main_info_str = match read_cache(&series_main_info_path).await {
            Ok(cache_string) => cache_string,
            Err(_) => {
                continue;
            }
        };

        let series_main_info = deserialize_json::<SeriesMainInformation>(&main_info_str)?;

        if series_main_info.status == "Ended" {
            clean_cache(&dir_entry.path()).await?;
        }
    }

    Ok(())
}

/// Cleans the cache of all running series depending on whether they are currently being aired or waiting for
/// their release dates
async fn clean_running_cache(running_status: &RunningStatus) -> anyhow::Result<()> {
    let mut read_dir = fs::read_dir(CACHER.get_cache_folder_path(CacheFolderType::Series)).await?;

    while let Some(dir_entry) = read_dir.next_entry().await? {
        let mut series_main_info_path = dir_entry.path();
        series_main_info_path.push(SERIES_MAIN_INFORMATION_FILENAME);

        let main_info_str = match read_cache(&series_main_info_path).await {
            Ok(cache_string) => cache_string,
            Err(_) => {
                continue;
            }
        };

        let mut series_episode_list_path = dir_entry.path();
        series_episode_list_path.push(EPISODE_LIST_FILENAME);

        let episode_list_cache_str = match read_cache(&series_episode_list_path).await {
            Ok(cache_string) => cache_string,
            Err(_) => {
                /* For Series to have no episode-list cache file, it's mostly because the series was loaded in the discover
                   page and thus episode-list were never loaded because it was not clicked. This will be treated as a aired
                   series since it was aired that's why it was in the discover page
                */
                if let RunningStatus::Aired = running_status {
                    clean_cache(&dir_entry.path()).await?;
                }
                continue;
            }
        };

        let series_main_info = deserialize_json::<SeriesMainInformation>(&main_info_str)?;

        if series_main_info.status != "Ended" {
            let mut series_episode_list_path = dir_entry.path();
            series_episode_list_path.push(EPISODE_LIST_FILENAME);

            let episode_list = EpisodeList::with_cache(&episode_list_cache_str)?;

            match running_status {
                RunningStatus::Aired => {
                    if episode_list.get_next_episode().is_some() {
                        clean_cache(&dir_entry.path()).await?;
                    }
                }
                RunningStatus::WaitingRelease => {
                    if episode_list.get_next_episode().is_none() {
                        clean_cache(&dir_entry.path()).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Removes the directory and it's contents at the given path
async fn clean_cache(path: &path::Path) -> anyhow::Result<()> {
    info!("cleaning cache: {}", path.display());
    fs::remove_dir_all(path)
        .await
        .unwrap_or_else(|err| error!("failed to clean cache for path {}: {}", path.display(), err));
    Ok(())
}
