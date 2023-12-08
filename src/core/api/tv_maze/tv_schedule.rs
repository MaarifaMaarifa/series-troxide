use super::deserialize_json;
use super::episodes_information::Episode;
use super::get_pretty_json_from_url;
use super::ApiError;

// replace "DATE" with an actual date in the format 2020-05-29
const SCHEDULE_ON_DATE_ADDRESS: &str = "https://api.tvmaze.com/schedule/web?date=DATE";

/// retrieves episodes aired on the current day at a particular country provided in ISO 3166-1
// replace "COUNTRY" with an actual country ISO in ISO 3166-1 format
const SCHEDULE_WITH_COUNTRY: &str = "https://api.tvmaze.com/schedule?country=COUNTRY";

// retrieves list of all future episodes known to TVmaze, regardless of their country
const FULL_SCHEDULE: &str = "https://api.tvmaze.com/schedule/full";

/// Retrieves episodes aired on a specific date through the provided optional &str
/// If None is supplied, it will default the the current day
pub async fn get_episodes_with_date(date: Option<&str>) -> Result<Vec<Episode>, ApiError> {
    let date = if let Some(date) = date { date } else { "" };

    let url = SCHEDULE_ON_DATE_ADDRESS.replace("DATE", date);

    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    deserialize_json::<Vec<Episode>>(&prettified_json).map(|mut episodes| {
        // deduplicating episodes that come from the same show
        episodes.dedup_by_key(|episode| episode.links.show.href.clone());
        episodes
    })
}

/// Retrieves episodes aired on the current day at a particular country provided in ISO 3166-1
pub async fn get_episodes_with_country(country_iso: &str) -> Result<Vec<Episode>, ApiError> {
    let url = SCHEDULE_WITH_COUNTRY.replace("COUNTRY", country_iso);

    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    deserialize_json::<Vec<Episode>>(&prettified_json).map(|mut episodes| {
        // deduplicating episodes that come from the same show
        episodes.dedup_by_key(|episode| episode.links.show.href.clone());
        episodes
    })
}

/// # Retrieves `full schedule`.
///
/// Full schedule is a list of all future episodes known to TVmaze, regardless of their country.
/// Returns the episodes in form of json string
pub async fn get_full_schedule() -> Result<String, ApiError> {
    get_pretty_json_from_url(FULL_SCHEDULE.to_string())
        .await
        .map_err(ApiError::Network)
}
