//! # Cache cleaning implementations
//!
//! Since the program keeps cache to avoid performing too many requests,
//! we need some form of cache cleaning so that we don't compromise the
//! ability of getting up to date information.

use std::path;

use crate::core::api::{deserialize_json, series_information::SeriesMainInformation};

use super::{
    episode_list::EpisodeList, read_cache, CacheFolderType, CACHER, EPISODE_LIST_FILENAME,
    SERIES_MAIN_INFORMATION_FILENAME,
};
use tokio::fs;
use tracing::{error, info};

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

/// Cleans the cache of all ended series
pub async fn clean_ended_series_cache() -> anyhow::Result<()> {
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
pub async fn clean_running_cache(running_status: RunningStatus) -> anyhow::Result<()> {
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
