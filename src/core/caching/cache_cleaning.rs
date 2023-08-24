//! # Cache cleaning implementations
//!
//! Since the program keeps cache to avoid performing too many requests,
//! we need some form of cache cleaning so that we don't compromise the
//! ability of getting up to date information.

use anyhow::Context;
use std::path;
use std::time;

use crate::core::{
    api::{deserialize_json, series_information::SeriesMainInformation},
    settings_config::CacheSettings,
};

use super::{
    episode_list::EpisodeList, read_cache, CacheFolderType, CACHER, EPISODE_LIST_FILENAME,
    SERIES_MAIN_INFORMATION_FILENAME,
};
use anyhow::bail;
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

/// Cleans the cache based on the expiration duration
///
/// If `None `is supplied, the directory is going to be cleaned immediately
/// If 'Some' is supplied, the directory duration is going to be compared and
/// if it exceeds the expiration time, it's going to be cleaned
pub async fn clean_cache(
    clean_type: CleanType,
    expiration_duration: Option<time::Duration>,
) -> anyhow::Result<()> {
    match clean_type {
        CleanType::Running(running_status) => {
            clean_running_cache(&running_status, expiration_duration).await?;
        }
        CleanType::Ended => {
            clean_ended_series_cache(expiration_duration).await?;
        }
    }
    Ok(())
}

/// Cleans all the cache based on the expiration duration set by the `CacheSettings`
pub async fn auto_clean(cache_settings: &CacheSettings) -> anyhow::Result<()> {
    info!("running cache autoclean...");

    info!("cleaning expired aired series cache");
    clean_cache(
        CleanType::Running(RunningStatus::Aired),
        Some(time::Duration::from_secs(
            cache_settings.aired_cache_clean_frequency as u64 * 24 * 60 * 60,
        )),
    )
    .await?;

    info!("cleaning expired waiting for release date series cache");
    clean_cache(
        CleanType::Running(RunningStatus::WaitingRelease),
        Some(time::Duration::from_secs(
            cache_settings.waiting_release_cache_clean_frequency as u64 * 24 * 60 * 60,
        )),
    )
    .await?;

    info!("cleaning expired ended series cache");
    clean_cache(
        CleanType::Ended,
        Some(time::Duration::from_secs(
            cache_settings.ended_cache_clean_frequency as u64 * 24 * 60 * 60,
        )),
    )
    .await?;

    Ok(())
}

// get `tokio::fs::ReadDir` of the series cache directory, creating the directory on the process if it does not exist
async fn get_series_cache_directory_path() -> anyhow::Result<fs::ReadDir> {
    let series_folder = CACHER.get_cache_folder_path(CacheFolderType::Series);
    Ok(loop {
        match fs::read_dir(&series_folder).await {
            Ok(read_dir) => break read_dir,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    fs::create_dir_all(&series_folder).await?;
                } else {
                    bail!(err);
                }
            }
        }
    })
}

/// Cleans the cache of all ended series
///
/// # Note
/// Cleans the cache based on the expiration duration
///
/// If `None `is supplied, the directory is going to be cleaned immediately
/// If 'Some' is supplied, the directory duration is going to be compared and
/// if it exceeds the expiration time, it's going to be cleaned
async fn clean_ended_series_cache(
    expiration_duration: Option<time::Duration>,
) -> anyhow::Result<()> {
    let mut read_dir = get_series_cache_directory_path().await?;

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

        if series_main_info.has_ended() {
            clean_directory_if_old(&dir_entry.path(), expiration_duration).await?;
        }
    }

    Ok(())
}

/// Cleans the cache of all running series depending on whether they are currently being aired or waiting for
/// their release dates
///
/// # Note
/// Cleans the cache based on the expiration duration
///
/// If `None `is supplied, the directory is going to be cleaned immediately
/// If 'Some' is supplied, the directory duration is going to be compared and
/// if it exceeds the expiration time, it's going to be cleaned
async fn clean_running_cache(
    running_status: &RunningStatus,
    expiration_duration: Option<time::Duration>,
) -> anyhow::Result<()> {
    let mut read_dir = get_series_cache_directory_path().await?;

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

        if !series_main_info.has_ended() {
            let mut series_episode_list_path = dir_entry.path();
            series_episode_list_path.push(EPISODE_LIST_FILENAME);

            let episode_list = EpisodeList::with_cache(&episode_list_cache_str)?;

            match running_status {
                RunningStatus::Aired => {
                    if episode_list.get_next_episode().is_some() {
                        clean_directory_if_old(&dir_entry.path(), expiration_duration).await?;
                    }
                }
                RunningStatus::WaitingRelease => {
                    if episode_list.get_next_episode().is_none() {
                        clean_directory_if_old(&dir_entry.path(), expiration_duration).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Cleans the directory based on the expiration duration
///
/// If `None `is supplied, the directory is going to be cleaned immediately
/// If 'Some' is supplied, the directory duration is going to be compared and
/// if it exceeds the expiration time, it's going to be cleaned
async fn clean_directory_if_old(
    directory_path: &path::Path,
    expiration_duration: Option<time::Duration>,
) -> anyhow::Result<()> {
    if let Some(expiration_duration) = expiration_duration {
        if get_directory_age(directory_path)? > expiration_duration {
            clean_cache_directory(directory_path).await?;
        }
    } else {
        clean_cache_directory(directory_path).await?;
    }

    Ok(())
}

fn get_directory_age(directory_path: &path::Path) -> anyhow::Result<time::Duration> {
    directory_path
        .metadata()
        .context("failed to get directory metadata")?
        .created()
        .context("failed to get directory creation time")?
        .elapsed()
        .context("failed to get directory age")
}

/// Removes the directory and it's contents at the given path
async fn clean_cache_directory(path: &path::Path) -> anyhow::Result<()> {
    info!("cleaning cache: {}", path.display());
    fs::remove_dir_all(path)
        .await
        .unwrap_or_else(|err| error!("failed to clean cache for path {}: {}", path.display(), err));
    Ok(())
}
