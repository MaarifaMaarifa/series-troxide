use super::deserialize_json;
use super::get_pretty_json_from_url;
use super::series_information::SeriesMainInformation;
use super::ApiError;

pub mod show_updates {
    use tokio::task::JoinHandle;

    use crate::core::caching;

    use super::*;

    pub enum UpdateTimestamp {
        Day,
        Week,
        Month,
    }

    impl From<UpdateTimestamp> for String {
        fn from(value: UpdateTimestamp) -> Self {
            match value {
                UpdateTimestamp::Day => "day".to_owned(),
                UpdateTimestamp::Week => "week".to_owned(),
                UpdateTimestamp::Month => "month".to_owned(),
            }
        }
    }

    type ShowUpdatesIndex = indexmap::IndexMap<String, u64>;

    // replace TIMESTAMP with one of the variants of UpdateTimestamp enum as String
    const SERIES_UPDATES_ADDRESS: &str = "https://api.tvmaze.com/updates/shows?since=TIMESTAMP";

    async fn get_show_update_index(
        update_timestamp: UpdateTimestamp,
    ) -> Result<ShowUpdatesIndex, ApiError> {
        let url = String::from(SERIES_UPDATES_ADDRESS)
            .replace("TIMESTAMP", &String::from(update_timestamp));

        let prettified_json = get_pretty_json_from_url(url)
            .await
            .map_err(ApiError::Network)?;

        deserialize_json(&prettified_json)
    }

    /// # Get shows updates
    ///
    /// This function takes update timestamp when the shows were last updated and an Option<usize>
    /// which specifies the amount of shows to be returned, supply None if you want all of them
    /// but be aware that they can be alot especially if you provide big timestamps.
    pub async fn get_show_updates(
        update_timestamp: UpdateTimestamp,
        series_number: Option<usize>,
    ) -> Result<Vec<SeriesMainInformation>, ApiError> {
        let show_updates_index = get_show_update_index(update_timestamp).await?;

        let mut series_ids: Vec<String> = show_updates_index.into_keys().collect();

        if let Some(len) = series_number {
            series_ids.truncate(len);
        };

        let handles: Vec<JoinHandle<Result<SeriesMainInformation, ApiError>>> = series_ids
            .into_iter()
            .map(|series_id| series_id.parse::<u32>().expect("Can't parse series id"))
            .map(|series_id| tokio::task::spawn(caching::get_series_main_info_with_id(series_id)))
            .collect();

        let mut series_infos = Vec::with_capacity(handles.len());
        for handle in handles {
            let series_info = handle.await.expect("failed to await all series updates")?;
            series_infos.push(series_info);
        }

        Ok(series_infos)
    }
}
