use super::*;

const EPISODE_INFORMATION_ADDRESS: &str =
    "https://api.tvmaze.com/shows/SERIES-ID/episodebynumber?season=SEASON&number=EPISODE";

#[derive(Debug, Deserialize, Clone)]
pub struct Episode {
    pub name: String,
    pub season: u32,
    pub number: Option<u32>,
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
        .map_err(ApiError::Network)?;

    deserialize_json(&prettified_json)
}
