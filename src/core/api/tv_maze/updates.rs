use super::deserialize_json;
use super::get_pretty_json_from_url;
use super::ApiError;

use std::collections::HashMap;

/// Retrieves all the shows update
const SERIES_UPDATES_ADDRESS: &str = "https://api.tvmaze.com/updates/shows";
/// Retrieves the shows update with last update duration filter, the filter goes at the end of url.
const SERIES_UPDATES_ADDRESS_FILTERED: &str = "https://api.tvmaze.com/updates/shows?since=";

/// A list of all shows in the TVmaze database and the timestamp when they were last updated.
/// Updating a direct or indirect child of a show will also mark the show itself as updated.
/// For example; creating, deleting or updating an episode or an episode's gallery item will
/// mark the episode's show as updated. It's possible to filter the resultset to only include
/// shows that have been updated in the past day (24 hours), week, or month.
pub async fn get_shows_updates_index(
    last_updated: Option<LastUpdated>,
) -> Result<HashMap<String, i64>, ApiError> {
    let url = if let Some(last_updated) = last_updated {
        format!("{}{}", SERIES_UPDATES_ADDRESS_FILTERED, last_updated)
    } else {
        SERIES_UPDATES_ADDRESS.to_string()
    };

    tracing::info!("fetching shows updates");

    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    deserialize_json(&prettified_json)
}

pub enum LastUpdated {
    Day,
    Week,
    Month,
}

impl std::fmt::Display for LastUpdated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            LastUpdated::Day => "day",
            LastUpdated::Week => "week",
            LastUpdated::Month => "month",
        };
        write!(f, "{}", str)
    }
}
