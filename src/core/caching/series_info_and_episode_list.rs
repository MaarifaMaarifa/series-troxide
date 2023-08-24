use tokio::fs;
use tokio::sync::mpsc;
use tracing::info;

use super::episode_list::EpisodeList;
use super::series_information::get_series_main_info_with_id;
use super::CACHER;
use crate::core::api::series_information::get_series_info_and_episode_list;

#[derive(Copy, Clone)]
enum MissingCache {
    Both,
    None,
    Series,
    EpisodeList,
}

pub struct SeriesInfoAndEpisodeList {
    series_ids: Vec<u32>,
    completion_signal_sender: mpsc::Sender<anyhow::Result<()>>,
}

impl SeriesInfoAndEpisodeList {
    pub fn new(series_ids: Vec<u32>) -> (Self, mpsc::Receiver<anyhow::Result<()>>) {
        let (tx, rx) = mpsc::channel(series_ids.len());
        (
            Self {
                series_ids,
                completion_signal_sender: tx,
            },
            rx,
        )
    }

    /// Caches `SeriesMainInformation` and `EpisodeList` for all the series supplied via their ids
    pub async fn run_full_caching(&self, report_progress: bool) -> anyhow::Result<()> {
        let handles: Vec<_> = self
            .series_ids
            .iter()
            .map(|series_id| {
                let series_id = *series_id;
                let sender = self.completion_signal_sender.clone();
                tokio::spawn(async move {
                    let res = Self::run_caching(series_id).await;
                    if report_progress {
                        sender
                            .send(res)
                            .await
                            .expect("failed to send completion signal to the receiver");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await?;
        }
        Ok(())
    }

    async fn run_caching(series_id: u32) -> anyhow::Result<()> {
        match Self::get_missing_cache(series_id).await? {
            MissingCache::None => {}
            MissingCache::Series => Self::cache_series_information(series_id).await?,
            MissingCache::EpisodeList => Self::cache_episode_list(series_id).await?,
            MissingCache::Both => {
                let series_info = get_series_info_and_episode_list(series_id).await?;
                let episode_list = series_info
                    .get_episode_list()
                    .expect("series info should have embedded episode list");

                let handle_1 = tokio::spawn({
                    let series_cache_path = CACHER.get_cache_file_path(
                        super::CacheFilePath::SeriesMainInformation(series_id),
                    );

                    let mut series_cache_folder = series_cache_path.clone();
                    series_cache_folder.pop();

                    fs::create_dir_all(&series_cache_folder)
                        .await
                        .expect("failed to create series cache directory");

                    info!("caching 'series information' for series id {}", series_id);

                    fs::write(
                        series_cache_path,
                        serde_json::to_string_pretty(&series_info)
                            .expect("series information should be serializable"),
                    )
                });

                let handle_2 = tokio::spawn({
                    let episode_cache_path = CACHER
                        .get_cache_file_path(super::CacheFilePath::SeriesEpisodeList(series_id));

                    let mut episodes_cache_folder = episode_cache_path.clone();
                    episodes_cache_folder.pop();

                    fs::create_dir_all(&episodes_cache_folder)
                        .await
                        .expect("failed to create series cache directory");

                    info!("caching 'episode list' for series id {}", series_id);

                    fs::write(
                        episode_cache_path,
                        serde_json::to_string_pretty(&episode_list)
                            .expect("series information should be serializable"),
                    )
                });

                handle_1.await??;
                handle_2.await??;
            }
        }
        Ok(())
    }

    async fn get_missing_cache(series_id: u32) -> anyhow::Result<MissingCache> {
        let episode_list_cache_path =
            CACHER.get_cache_file_path(super::CacheFilePath::SeriesEpisodeList(series_id));
        let series_info_cache_path =
            CACHER.get_cache_file_path(super::CacheFilePath::SeriesMainInformation(series_id));

        let missing_cache = &mut [None; 2];

        if !fs::try_exists(episode_list_cache_path).await? {
            missing_cache[0] = Some(MissingCache::EpisodeList);
        }
        if !fs::try_exists(series_info_cache_path).await? {
            missing_cache[1] = Some(MissingCache::Series);
        }

        Ok(if missing_cache.iter().all(|x| x.is_some()) {
            MissingCache::Both
        } else if missing_cache.iter().all(|x| x.is_none()) {
            MissingCache::None
        } else {
            missing_cache
                .iter()
                .find_map(|x| *x)
                .expect("atleast one missing cache type should exist")
        })
    }

    async fn cache_episode_list(series_id: u32) -> anyhow::Result<()> {
        // Since we just care when the episode list is cached, we discard the
        // returned EpisodeList.
        let _ = EpisodeList::new(series_id).await?;
        Ok(())
    }

    async fn cache_series_information(series_id: u32) -> anyhow::Result<()> {
        // Since we just care when the series info is cached, we discard the
        // returned series information.
        let _ = get_series_main_info_with_id(series_id).await?;
        Ok(())
    }
}
