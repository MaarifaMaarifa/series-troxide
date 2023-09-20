use super::series_information::SeriesMainInformation;
use super::{deserialize_json, ApiError};

// For tvdb, the link should look like https://api.tvmaze.com/lookup/shows?thetvdb=81189
// For imdb, the link should look like https://api.tvmaze.com/lookup/shows?imdb=tt0944947
const SHOW_LOOKUP_ADDRESS: &str = "https://api.tvmaze.com/lookup/shows?";

/// Id to be used on show lookup
pub enum Id {
    Imdb(String),
    Tvdb(u32),
}

/// Looks up a show if available on TVmaze based on the supplied id and returns it's `SeriesMainInformation` if available
pub async fn show_lookup(show_id: Id) -> Result<Option<SeriesMainInformation>, ApiError> {
    let url = match show_id {
        Id::Imdb(imdb_id) => format!("{}{}{}", SHOW_LOOKUP_ADDRESS, "imdb=", imdb_id),
        Id::Tvdb(tvdb_id) => format!("{}{}{}", SHOW_LOOKUP_ADDRESS, "thetvdb=", tvdb_id),
    };

    let pretty_json_str = super::get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    // handling the case when the show is not found
    if serde_json::from_str::<Option<()>>(&pretty_json_str).is_ok() {
        return Ok(None);
    }

    Ok(Some(deserialize_json(&pretty_json_str)?))
}
