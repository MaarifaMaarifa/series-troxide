use super::*;

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

// TODO: Creating a release profile that get's rid of all this boilerplate

pub async fn get_series_main_info_with_url(url: String) -> Result<SeriesMainInformation, ApiError> {
    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    deserialize_json(&prettified_json)
}

pub async fn get_series_main_info_with_id(
    series_id: u32,
) -> Result<SeriesMainInformation, ApiError> {
    get_series_main_info_with_url(format!("{}{}", SERIES_INFORMATION_ADDRESS, series_id)).await
}

pub async fn get_series_main_info_with_ids(series_ids: Vec<String>) -> Vec<SeriesMainInformation> {
    let handles: Vec<_> = series_ids
        .iter()
        .map(|id| tokio::spawn(get_series_main_info_with_id(id.parse().unwrap())))
        .collect();

    let mut series_infos = Vec::with_capacity(handles.len());
    for handle in handles {
        let series_info = handle.await.unwrap().unwrap();
        series_infos.push(series_info);
    }
    series_infos
}
