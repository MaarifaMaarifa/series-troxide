// use anyhow::bail;
// use tokio::task::JoinHandle;

use super::*;

// The series name goes after the equals sign
const SERIES_SEARCH_ADDRESS: &str = "https://api.tvmaze.com/search/shows?q=";

#[derive(Debug, Deserialize, Clone)]
pub struct SeriesSearchResult {
    pub show: Show,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Show {
    pub id: u32,
    pub name: String,
    pub premiered: Option<String>,
    pub genres: Vec<String>,
    pub image: Option<Image>,
}

pub async fn search_series(series_name: String) -> Result<Vec<SeriesSearchResult>, ApiError> {
    let url = format!("{}{}", SERIES_SEARCH_ADDRESS, series_name);

    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    deserialize_json(&prettified_json)
}
