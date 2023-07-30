//! Perform different operations on the database series

use super::series_information;
use crate::core::{
    api::series_information::SeriesMainInformation,
    database::{self, Series},
};
use lazy_static::lazy_static;

lazy_static! {
    static ref TRACKED_SERIES_INFORMATIONS_REQUEST_LOCK: tokio::sync::Mutex<()> =
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

    pub async fn get_untracked_series_informations(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        let handles: Vec<_> = self
            .get_untracked_series_ids()
            .iter()
            .map(|id| {
                let id = id.parse().expect("could not parse series id");
                tokio::spawn(series_information::get_series_main_info_with_id(id))
            })
            .collect();

        let mut series_informations = Vec::with_capacity(handles.len());
        for handle in handles {
            series_informations.push(handle.await??)
        }

        Ok(series_informations)
    }

    pub async fn get_tracked_series_informations(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        // Since diferrent methods can end up calling this same method, they will end up doing
        // multiple unecessary api request if the data is not cached, this lock makes the first
        // code to call this method to do all the work first and the other methods will eventually
        // read from the cache
        let _ = TRACKED_SERIES_INFORMATIONS_REQUEST_LOCK.lock().await;

        let handles: Vec<_> = self
            .get_tracked_series_ids()
            .iter()
            .map(|id| {
                let id = id.parse().expect("could not parse series id");
                tokio::spawn(series_information::get_series_main_info_with_id(id))
            })
            .collect();

        let mut series_informations = Vec::with_capacity(handles.len());
        for handle in handles {
            series_informations.push(handle.await??)
        }

        Ok(series_informations)
    }

    pub async fn get_running_tracked_series_informations(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        Ok(self
            .get_tracked_series_informations()
            .await?
            .into_iter()
            .filter(|series_info| series_info.status != "Ended")
            .collect())
    }

    pub async fn get_ended_tracked_series_informations(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        Ok(self
            .get_tracked_series_informations()
            .await?
            .into_iter()
            .filter(|series_info| series_info.status == "Ended")
            .collect())
    }

    pub async fn get_waiting_release_series_informations(
        &self,
    ) -> anyhow::Result<Vec<SeriesMainInformation>> {
        let series_infos = self.get_running_tracked_series_informations().await?;

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
            if episode_list.get_next_episode().is_none() {
                waiting_releases_series_infos.push(series_info)
            }
        }
        Ok(waiting_releases_series_infos)
    }

    pub async fn get_upcoming_release_series_informations_and_episodes(
        &self,
    ) -> anyhow::Result<Vec<(SeriesMainInformation, super::episode_list::EpisodeList)>> {
        let series_infos = self.get_running_tracked_series_informations().await?;
        let mut waiting_releases_series_infos = Vec::with_capacity(series_infos.len());
        for series_info in series_infos {
            let episode_list = super::episode_list::EpisodeList::new(series_info.id).await?;
            if episode_list.get_next_episode().is_some() {
                waiting_releases_series_infos.push((series_info, episode_list))
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
