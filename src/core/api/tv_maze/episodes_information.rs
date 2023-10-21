use chrono::Local;

use super::{series_information::SeriesMainInformation, *};

const EPISODE_INFORMATION_ADDRESS: &str =
    "https://api.tvmaze.com/shows/SERIES-ID/episodebynumber?season=SEASON&number=EPISODE";

const EPISODE_LIST_ADDRESS: &str = "https://api.tvmaze.com/shows/SERIES-ID/episodes";

/// # An `Episode` data according to the TVmaze api
///
/// This data discribes an episode found in a season of a particular series
///
/// ## Note

/// There are two important fields to pay attention to
///
/// ### show
///
/// This field carries an `Option<SeriesMainInformation>`. This field becomes the `Some`
/// variant when the episode is retrieved as an local aired episode which are country
/// specific. [link](https://www.tvmaze.com/api#schedule)
///
/// ### embedded
///
/// This field carries an `Option<Embedded>` where `Embedded` field carries `SeriesInformation`.
/// This field becomes the `Some` variant when the episode is retrieved as a global aired episode.
/// [link](https://www.tvmaze.com/api#web-schedule)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Episode {
    pub name: String,
    pub season: u32,
    pub number: Option<u32>,
    pub runtime: Option<u32>,
    pub airdate: Option<String>,
    pub airtime: String, // can be empty
    pub airstamp: Option<String>,
    pub rating: Rating,
    pub image: Option<Image>,
    pub summary: Option<String>,
    /// Local aired episodes normally have this field as `Some`
    pub show: Option<SeriesMainInformation>,
    #[serde(rename = "_links")]
    pub links: Links,
    /// Global aired episodes normally have this field as `Some`
    #[serde(rename = "_embedded")]
    pub embedded: Option<Embedded>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Embedded {
    pub show: SeriesMainInformation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Links {
    pub show: Show,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Show {
    pub href: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EpisodeDateError {
    #[error("no date was found in the episode")]
    NotFound,

    #[error("failed to parse the date")]
    Parse(chrono::ParseError),
}

impl Episode {
    pub fn date_naive(&self) -> Result<chrono::NaiveDate, EpisodeDateError> {
        Ok(self.local_date_time()?.date_naive())
    }

    pub fn local_date_time(&self) -> Result<chrono::DateTime<Local>, EpisodeDateError> {
        let date_time_str = self.airstamp.as_ref().ok_or(EpisodeDateError::NotFound)?;
        Ok(chrono::DateTime::parse_from_rfc3339(date_time_str)
            .map_err(EpisodeDateError::Parse)?
            .with_timezone(&Local))
    }
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

pub async fn get_episode_list(series_id: u32) -> Result<(Vec<Episode>, String), ApiError> {
    let url = EPISODE_LIST_ADDRESS.replace("SERIES-ID", &series_id.to_string());
    let prettified_json = get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)?;

    Ok((deserialize_json(&prettified_json)?, prettified_json))
}
