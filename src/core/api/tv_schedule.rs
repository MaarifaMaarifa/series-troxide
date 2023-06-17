use super::deserialize_json;
use super::episodes_information::Episode;
use super::get_pretty_json_from_url;
use super::ApiError;

// replace "DATE" with an actual date in the format 2020-05-29
const SCHEDULE_ON_DATE_ADDRESS: &str = "https://api.tvmaze.com/schedule/web?date=DATE";

/// Retrieves episodes aired on a specific date through the provided optional &str
/// If None is supplied, it will default the the current day
pub async fn get_episodes_with_date(date: Option<&str>) -> Result<Vec<Episode>, ApiError> {
    let date = if let Some(date) = date { date } else { "" };

    let url = SCHEDULE_ON_DATE_ADDRESS.replace("DATE", date);

    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    deserialize_json(&prettified_json)
}
