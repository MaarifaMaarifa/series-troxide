use super::*;

// replace the word SERIES-ID with the actual series id
const SEASONS_LIST_ADDRESS: &str = "https://api.tvmaze.com/shows/SERIES-ID/seasons";

#[derive(Debug, Deserialize, Clone)]
pub struct Season {
    pub number: u32,
    #[serde(rename = "episodeOrder")]
    pub episode_order: Option<u32>,
    #[serde(rename = "premiereDate")]
    pub premiere_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}

pub async fn get_seasons_list(series_id: u32) -> Result<Vec<Season>, ApiError> {
    let url = SEASONS_LIST_ADDRESS.replace("SERIES-ID", &series_id.to_string());
    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    deserialize_json(&prettified_json)
}
