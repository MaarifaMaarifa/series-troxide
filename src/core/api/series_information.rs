use super::*;
use std::hash::{Hash, Hasher};

// The series id goes after the last slash(append at the end of the string)
const SERIES_INFORMATION_ADDRESS: &str = "https://api.tvmaze.com/shows/";

#[derive(Debug, Deserialize, Clone)]
pub struct SeriesMainInformation {
    pub id: u32,
    pub name: String,
    pub language: Option<String>,
    pub genres: Vec<String>,
    pub status: String,
    #[serde(rename = "averageRuntime")]
    pub average_runtime: Option<u32>,
    pub premiered: Option<String>,
    pub ended: Option<String>,
    pub rating: Rating,
    pub network: Option<Network>,
    #[serde(rename = "webChannel")]
    pub web_channel: Option<WebChannel>,
    pub summary: Option<String>,
    pub image: Option<Image>,
}

impl PartialEq for SeriesMainInformation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SeriesMainInformation {}

impl Hash for SeriesMainInformation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebChannel {
    pub name: String,
    #[serde(rename = "officialSite")]
    pub official_site: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Network {
    pub name: String,
    pub country: Country,
    #[serde(rename = "officialSite")]
    pub official_site_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Country {
    pub name: String,
}

pub async fn get_series_main_info_with_url(url: String) -> Result<String, ApiError> {
    get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)
}

pub async fn get_series_main_info_with_id(series_id: u32) -> Result<String, ApiError> {
    get_series_main_info_with_url(format!("{}{}", SERIES_INFORMATION_ADDRESS, series_id)).await
}
