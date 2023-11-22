//! Perform different operations on the database series

use super::{episode_list::EpisodeReleaseTime, series_information};
use crate::core::{
    api::tv_maze::{episodes_information::Episode, series_information::SeriesMainInformation},
    database::{self, Series},
};
use lazy_static::lazy_static;

lazy_static! {
    static ref TRACKED_SERIES_INFORMATION_REQUEST_LOCK: tokio::sync::Mutex<()> =
        tokio::sync::Mutex::new(());
}

pub struct SeriesList {
    series_list: Vec<(String, Series)>,
}

impl SeriesList {
    pub fn new() -> Self {
        Self {
            series_list: database::DB.get_ids_and_series(),
        }
    }

    pub fn get_tracked_series_ids(&self) -> Vec<&str> {
        self.series_list
            .iter()
            .filter(|(_, series)| series.is_tracked())
            .map(|(id, _)| id.as_str())
            .collect()
    }

    pub fn get_untracked_series_ids(&self) -> Vec<&str> {
        self.series_list
            .iter()
            .filter(|(_, series)| !series.is_tracked())
            .map(|(id, _)| id.as_str())
            .collect()
    }

    pub async fn get_untracked_series_information(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        let untracked_ids: Vec<u32> = self
            .get_untracked_series_ids()
            .into_iter()
            .map(|id| id.parse().expect("could not parse series id"))
            .collect();

        let (series_info_and_episode_list, _) =
            super::series_info_and_episode_list::SeriesInfoAndEpisodeList::new(
                untracked_ids.clone(),
            );

        // Fetching cache more efficiently if they dont exist
        series_info_and_episode_list.run_full_caching(false).await?;

        let handles: Vec<_> = untracked_ids
            .iter()
            .map(|id| tokio::spawn(series_information::get_series_main_info_with_id(*id)))
            .collect();

        let mut series_information = Vec::with_capacity(handles.len());
        for handle in handles {
            series_information.push(handle.await??)
        }

        Ok(series_information)
    }

    pub async fn get_tracked_series_information(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        // Since different methods can end up calling this same method, they will end up doing
        // multiple unnecessary api request if the data is not cached, this lock makes the first
        // code to call this method to do all the work first and the other methods will eventually
        // read from the cache
        let _ = TRACKED_SERIES_INFORMATION_REQUEST_LOCK.lock().await;

        let tracked_ids: Vec<u32> = self
            .get_tracked_series_ids()
            .into_iter()
            .map(|id| id.parse().expect("could not parse series id"))
            .collect();

        let (series_info_and_episode_list, _) =
            super::series_info_and_episode_list::SeriesInfoAndEpisodeList::new(tracked_ids.clone());

        // Fetching cache more efficiently if they dont exist
        series_info_and_episode_list.run_full_caching(false).await?;

        let handles: Vec<_> = tracked_ids
            .iter()
            .map(|id| tokio::spawn(series_information::get_series_main_info_with_id(*id)))
            .collect();

        let mut series_information = Vec::with_capacity(handles.len());
        for handle in handles {
            series_information.push(handle.await??)
        }

        Ok(series_information)
    }

    /// Gets the series information of all the series in the database
    pub async fn get_all_series_information(&self) -> anyhow::Result<Vec<SeriesMainInformation>> {
        let _ = TRACKED_SERIES_INFORMATION_REQUEST_LOCK.lock().await;

        let handles: Vec<_> = self
            .series_list
            .iter()
            .map(|(id, _)| {
                let id = id.parse().expect("could not parse series id");
                tokio::spawn(series_information::get_series_main_info_with_id(id))
            })
            .collect();

        let mut series_information = Vec::with_capacity(handles.len());
        for handle in handles {
            series_information.push(handle.await??)
        }

        Ok(series_information)
    }

    pub async fn get_running_tracked_series_information(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        Ok(self
            .get_tracked_series_information()
            .await?
            .into_iter()
            .filter(|series_info| series_info.status != "Ended")
            .collect())
    }

    pub async fn get_ended_tracked_series_information(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        Ok(self
            .get_tracked_series_information()
            .await?
            .into_iter()
            .filter(|series_info| series_info.status == "Ended")
            .collect())
    }

    pub async fn get_waiting_release_series_information(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        let series_infos = self.get_running_tracked_series_information().await?;

        let mut episode_list_handles = Vec::with_capacity(series_infos.len());
        for series_info in series_infos.iter() {
            episode_list_handles.push(tokio::spawn(super::episode_list::EpisodeList::new(
                series_info.id,
            )))
        }

        let mut waiting_releases_series_infos = Vec::with_capacity(series_infos.len());
        for (handle, series_info) in episode_list_handles
            .into_iter()
            .zip(series_infos.into_iter())
        {
            let episode_list = handle.await??;
            if episode_list.get_next_episode_to_air().is_none() {
                waiting_releases_series_infos.push(series_info)
            }
        }
        Ok(waiting_releases_series_infos)
    }

    pub async fn get_upcoming_release_series_information_and_episodes(
        &self,
    ) -> anyhow::Result<Vec<(SeriesMainInformation, Episode, EpisodeReleaseTime)>> {
        let series_infos = self.get_running_tracked_series_information().await?;
        let mut waiting_releases_series_infos = Vec::with_capacity(series_infos.len());

        let handles: Vec<_> = series_infos
            .iter()
            .map(|series_info| tokio::spawn(super::episode_list::EpisodeList::new(series_info.id)))
            .collect();

        for (handle, series_info) in handles.into_iter().zip(series_infos.into_iter()) {
            let episode_list = handle.await??;
            if let Some((next_episode, release_time)) =
                episode_list.get_next_episode_to_air_and_time()
            {
                waiting_releases_series_infos.push((
                    series_info,
                    next_episode.to_owned(),
                    release_time,
                ))
            }
        }
        Ok(waiting_releases_series_infos)
    }
}

impl Default for SeriesList {
    fn default() -> Self {
        Self::new()
    }
}
