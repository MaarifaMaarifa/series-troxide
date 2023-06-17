use super::deserialize_json;
use super::episodes_information::Episode;
use super::get_pretty_json_from_url;
use super::ApiError;

// replace "DATE" with an actual date in the format 2020-05-29
const SCHEDULE_ON_DATE_ADDRESS: &str = "https://api.tvmaze.com/schedule/web?date=DATE";

pub async fn get_episodes_with_date(date: &str) -> Result<Vec<Episode>, ApiError> {
    let url = SCHEDULE_ON_DATE_ADDRESS.replace("DATE", date);

    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(|err| ApiError::Network(err))?;

    deserialize_json(&prettified_json)
}
