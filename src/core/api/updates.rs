use super::deserialize_json;
use super::get_pretty_json_from_url;
use super::series_information::{get_series_main_info_with_id, SeriesMainInformation};
use super::ApiError;

pub mod show_updates {
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
            .map_err(|err| ApiError::Network(err))?;

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

        let mut series_infos = Vec::with_capacity(series_ids.len());
        for series_id in series_ids {
            let series_id: u32 = series_id
                .parse()
                .expect("Unable to convert series id to a u32");

            let series_info = get_series_main_info_with_id(series_id).await?;
            series_infos.push(series_info);
        }

        Ok(series_infos)
    }
}
