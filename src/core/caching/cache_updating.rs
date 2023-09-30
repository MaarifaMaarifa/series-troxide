//! # Cache updating implementations
//!
//! Since the program keeps cache to avoid performing too many requests,
//! we need some form of cache updating so that we stay up to date.

use std::path;
use std::time;

use anyhow::Context;
use tokio::fs;
use tracing::{error, info, warn};

use super::series_info_and_episode_list::SeriesInfoAndEpisodeList;
use super::{CacheFolderType, CACHER};
use crate::core::api::tv_maze::updates::get_shows_updates_index;
use crate::core::database::DB;

async fn get_all_series_cache_directories(
) -> anyhow::Result<Vec<(String, path::PathBuf, time::Duration)>> {
    let series_cache_folder = CACHER.get_cache_folder_path(CacheFolderType::Series);

    let mut read_dir = fs::read_dir(series_cache_folder)
        .await
        .context("failed to read series cache directory")?;

    let mut series_dirs = vec![];
    while let Some(dir_entry) = read_dir
        .next_entry()
        .await
        .context("failed to read a series directory entry")?
    {
        let dir_path = dir_entry.path();
        let duration = dir_path
            .metadata()
            .context("failed to get series cache directory metadata")?
            .created()
            .context("failed to get creating time of a series cache directory")?
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .context("system clock failure when determining series cache folder creation")?;

        let series_id = dir_path
            .file_name()
            .expect("invalid series cache path")
            .to_string_lossy()
            .to_string();
        series_dirs.push((series_id, dir_path, duration))
    }
    Ok(series_dirs)
}

pub async fn update_cache() -> anyhow::Result<()> {
    if !should_update().await? {
        return Ok(());
    }

    info!("updating series cache...");

    let updates_index = get_shows_updates_index(None).await?;

    let series_cache_directories = get_all_series_cache_directories().await?;

    let mut handles = Vec::with_capacity(series_cache_directories.len());
    for (series_id, path, cache_timestamp) in series_cache_directories {
        let time_stamp = updates_index.get(&series_id).copied();

        let handle = tokio::spawn(async move {
            if let Some(time_stamp) = time_stamp {
                let update_timestamp = time::Duration::from_secs(time_stamp as u64);

                if update_timestamp > cache_timestamp {
                    clean_cache_directory(&path).await;

                    // Caching the series if it's in the database
                    let series_id: u32 = series_id.parse().expect("series id should be parsable");
                    if DB.get_series(series_id).is_some() {
                        SeriesInfoAndEpisodeList::cache_series(series_id)
                            .await
                            .unwrap_or_else(|err| {
                                error!("failed to cache series with id '{}': {}", series_id, err)
                            });
                    }
                }
            } else {
                warn!(
                    "series cache with id '{}' not in updates, cleaning it anyways",
                    series_id
                );
                clean_cache_directory(&path).await
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("failed to join cache updates handles");
    }

    record_last_update().await?;

    info!("updating series cache complete!");

    Ok(())
}

const LAST_UPDATE_FILENAME: &str = "last-cache-update";

fn get_last_update_filepath() -> path::PathBuf {
    let mut last_update_file = CACHER.get_root_cache_path().to_owned();
    last_update_file.push(LAST_UPDATE_FILENAME);
    last_update_file
}

fn duration_since_epoch() -> anyhow::Result<time::Duration> {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .context("system clock failure when determining current time")
}

/// Whether cache should be updated or not
///
/// Checks if a day has passed since the last cache update and returns `true`,
/// Otherwise the opposite
async fn should_update() -> anyhow::Result<bool> {
    let last_update_file = get_last_update_filepath();

    let current_timestamp = duration_since_epoch()?;

    let last_update_timestamp: u64 = match fs::read_to_string(last_update_file).await {
        Ok(content) => match content.parse() {
            Ok(val) => val,
            Err(err) => {
                error!("failed to parse 'last-cache-update' file: {}", err);
                warn!("assuming a day has lasted since last cache update");
                return Ok(true);
            }
        },
        Err(err) => {
            error!("could not read 'last-cache-update' file: {}", err);
            warn!("assuming a day has lasted since last cache update");
            return Ok(true);
        }
    };

    let last_update_timestamp = time::Duration::from_secs(last_update_timestamp);

    Ok((current_timestamp - last_update_timestamp) > time::Duration::from_secs(60 * 60 * 24))
}

async fn record_last_update() -> anyhow::Result<()> {
    let last_update_file = get_last_update_filepath();

    let current_timestamp = duration_since_epoch()?;

    fs::write(last_update_file, current_timestamp.as_secs().to_string())
        .await
        .context("failed to write 'last-cache-update' file")
}

/// Removes the directory and it's contents at the given path
async fn clean_cache_directory(path: &path::Path) {
    info!("cleaning cache: {}", path.display());
    fs::remove_dir_all(path)
        .await
        .unwrap_or_else(|err| error!("failed to clean cache for path {}: {}", path.display(), err));
}
