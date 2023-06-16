use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("network error during request")]
    Network(reqwest::Error),
    #[error("tvmaze api error when deserializing json: unexpected '{0}'")]
    Deserialization(String, serde_json::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rating {
    pub average: Option<f32>,
}

/// Loads the image from the provided url
pub async fn load_image(image_url: String) -> Option<Vec<u8>> {
    loop {
        match reqwest::get(&image_url).await {
            Ok(response) => {
                if let Ok(bytes) = response.bytes().await {
                    let bytes: Vec<u8> = bytes.into();
                    break Some(bytes);
                }
            }
            Err(ref err) => {
                if err.is_request() {
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                } else {
                    break None;
                }
            }
        }
    }
}

fn deserialize_json<'a, T: serde::Deserialize<'a>>(
    prettified_json: &'a str,
) -> Result<T, ApiError> {
    serde_json::from_str::<T>(&prettified_json).map_err(|err| {
        let line_number = err.line() - 1;

        let mut errored_line = String::new();
        prettified_json
            .lines()
            .skip(line_number)
            .take(1)
            .for_each(|line| errored_line = line.to_owned());
        ApiError::Deserialization(errored_line, err)
    })
}

/// Requests text response from the provided url
async fn get_pretty_json_from_url(url: String) -> Result<String, reqwest::Error> {
    let text = reqwest::get(url).await?.text().await?;
    Ok(json::stringify_pretty(json::parse(&text).unwrap(), 1))
}

#[derive(Debug, Deserialize, Clone)]
pub struct Image {
    #[serde(rename = "original")]
    pub original_image_url: String,
    #[serde(rename = "medium")]
    pub medium_image_url: String,
}

pub mod series_searching {
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
            .map_err(|err| ApiError::Network(err))?;

        deserialize_json(&prettified_json)
    }
}

pub mod series_information {
    use super::*;

    // The series id goes after the last slash(append at the end of the string)
    const SERIES_INFORMATION_ADDRESS: &str = "https://api.tvmaze.com/shows/";

    #[derive(Debug, Deserialize, Clone)]
    pub struct SeriesMainInformation {
        pub name: String,
        pub language: String,
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

    // TODO: Refactor code repetition in both of these functions
    // TODO: Creating a release profile that get's rid of all this boilerplate

    pub async fn get_series_main_info_with_url(
        url: String,
    ) -> Result<SeriesMainInformation, ApiError> {
        let prettified_json = get_pretty_json_from_url(url)
            .await
            .map_err(|err| ApiError::Network(err))?;

        deserialize_json(&prettified_json)
    }

    pub async fn get_series_main_info_with_id(
        series_id: u32,
    ) -> Result<SeriesMainInformation, ApiError> {
        get_series_main_info_with_url(format!("{}{}", SERIES_INFORMATION_ADDRESS, series_id)).await
    }
}

pub mod seasons_list {
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
            .map_err(|err| ApiError::Network(err))?;

        deserialize_json(&prettified_json)
    }
}

pub mod episodes_information {
    use super::*;

    const EPISODE_INFORMATION_ADDRESS: &str =
        "https://api.tvmaze.com/shows/SERIES-ID/episodebynumber?season=SEASON&number=EPISODE";

    #[derive(Debug, Deserialize, Clone)]
    pub struct Episode {
        pub name: String,
        pub season: u32,
        pub number: u32,
        pub runtime: Option<u32>,
        pub airdate: Option<String>,
        pub airtime: String, // can be empty
        pub airstamp: String,
        pub rating: Rating,
        pub image: Option<Image>,
        pub summary: Option<String>,
        #[serde(rename = "_links")]
        pub links: Links,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct Links {
        pub show: Show,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct Show {
        pub href: String,
    }

    pub async fn get_episode_information(
        series_id: u32,
        season: u32,
        episode: u32,
    ) -> Result<Episode, ApiError> {
        let url = EPISODE_INFORMATION_ADDRESS.replace("SERIES-ID", &series_id.to_string());
        let url = url.replace("SEASON", &season.to_string());
        let url = url.replace("EPISODE", &episode.to_string());

        let prettified_json = get_pretty_json_from_url(url)
            .await
            .map_err(|err| ApiError::Network(err))?;

        deserialize_json(&prettified_json)
    }
}

pub mod tv_schedule {
    use super::{
        deserialize_json, episodes_information::Episode, get_pretty_json_from_url, ApiError,
    };

    // replace "DATE" with an actual date in the format 2020-05-29
    const SCHEDULE_ON_DATE_ADDRESS: &str = "https://api.tvmaze.com/schedule/web?date=DATE";

    pub async fn get_episodes_with_date(date: &str) -> Result<Vec<Episode>, ApiError> {
        let url = SCHEDULE_ON_DATE_ADDRESS.replace("DATE", date);

        let prettified_json = get_pretty_json_from_url(url)
            .await
            .map_err(|err| ApiError::Network(err))?;

        deserialize_json(&prettified_json)
    }
}
