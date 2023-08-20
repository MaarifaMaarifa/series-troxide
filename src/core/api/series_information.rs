use super::*;
use serde::Serialize;
use std::hash::{Hash, Hasher};

// The series id goes after the last slash(append at the end of the string)
const SERIES_INFORMATION_ADDRESS: &str = "https://api.tvmaze.com/shows/";

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Genre {
    Romance,
    Drama,
    Music,
    Action,
    Fantasy,
    ScienceFiction,
    Horror,
    Thriller,
    Crime,
    Adventure,
    Comedy,
    Adult,
    Anime,
    Children,
    Family,
    Food,
    Nature,
    Supernatural,
    Western,
    Espionage,
    Mystery,
    Other,
}

impl From<&str> for Genre {
    fn from(value: &str) -> Self {
        match value {
            "Romance" => Self::Romance,
            "Drama" => Self::Drama,
            "Music" => Self::Music,
            "Action" => Self::Action,
            "Fantasy" => Self::Fantasy,
            "Science-Fiction" => Self::ScienceFiction,
            "Horror" => Self::Horror,
            "Thriller" => Self::Thriller,
            "Crime" => Self::Crime,
            "Adventure" => Self::Adventure,
            "Comedy" => Self::Comedy,
            "Adult" => Self::Adult,
            "Anime" => Self::Anime,
            "Children" => Self::Children,
            "Family" => Self::Family,
            "Food" => Self::Food,
            "Nature" => Self::Nature,
            "Supernatural" => Self::Supernatural,
            "Western" => Self::Western,
            "Espionage" => Self::Espionage,
            "Mystery" => Self::Mystery,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Romance => "Romance",
            Self::Drama => "Drama",
            Self::Music => "Music",
            Self::Action => "Action",
            Self::Fantasy => "Fantasy",
            Self::ScienceFiction => "Science-Fiction",
            Self::Horror => "Horror",
            Self::Thriller => "Thriller",
            Self::Crime => "Crime",
            Self::Adventure => "Adventure",
            Self::Comedy => "Comedy",
            Self::Adult => "Adult",
            Self::Anime => "Anime",
            Self::Children => "Children",
            Self::Family => "Family",
            Self::Food => "Food",
            Self::Nature => "Nature",
            Self::Supernatural => "Supernatural",
            Self::Western => "Western",
            Self::Espionage => "Espionage",
            Self::Mystery => "Mystery",
            Self::Other => "Other",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ShowNetwork {
    Fox,
    TheCW,
    BbcOne,
    Nbc,
    Abc,
    Hbo,
    Cbs,
    Other,
}

impl From<&str> for ShowNetwork {
    fn from(value: &str) -> Self {
        match value {
            "FOX" => Self::Fox,
            "The CW" => Self::TheCW,
            "BBC One" => Self::BbcOne,
            "MSNBC" => Self::Nbc,
            "ABC" => Self::Abc,
            "HBO" => Self::Hbo,
            "CBS" => Self::Cbs,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for ShowNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let network_str = match self {
            ShowNetwork::Fox => "FOX",
            ShowNetwork::TheCW => "The CW",
            ShowNetwork::BbcOne => "BBC One",
            ShowNetwork::Nbc => "MSNBC",
            ShowNetwork::Abc => "ABC",
            ShowNetwork::Hbo => "HBO",
            ShowNetwork::Cbs => "CBS",
            ShowNetwork::Other => "Other",
        };
        write!(f, "{}", network_str)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ShowWebChannel {
    Netflix,
    Other,
}

impl From<&str> for ShowWebChannel {
    fn from(value: &str) -> Self {
        match value {
            "Netflix" => Self::Netflix,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for ShowWebChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let channel_str = match self {
            ShowWebChannel::Netflix => "Netflix",
            ShowWebChannel::Other => "Other",
        };
        write!(f, "{}", channel_str)
    }
}

#[derive(PartialEq)]
pub enum ShowStatus {
    Running,
    Ended,
    ToBeDetermined,
    InDevelopment,
    Other,
}

impl From<&str> for ShowStatus {
    fn from(value: &str) -> Self {
        match value {
            "Running" => Self::Running,
            "Ended" => Self::Ended,
            "To Be Determined" => Self::ToBeDetermined,
            "In Development" => Self::InDevelopment,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for ShowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match self {
            ShowStatus::Running => "Running",
            ShowStatus::Ended => "Ended",
            ShowStatus::ToBeDetermined => "To Be Determined",
            ShowStatus::InDevelopment => "In Development",
            ShowStatus::Other => "Other",
        };
        write!(f, "{}", status_str)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl SeriesMainInformation {
    pub fn get_genres(&self) -> Vec<Genre> {
        self.genres
            .iter()
            .map(|genre| Genre::from(genre.as_str()))
            .collect()
    }

    pub fn get_status(&self) -> ShowStatus {
        ShowStatus::from(self.status.as_str())
    }

    pub fn get_network(&self) -> Option<ShowNetwork> {
        self.network
            .as_ref()
            .map(|network| ShowNetwork::from(network.name.as_str()))
    }

    pub fn get_webchannel(&self) -> Option<ShowWebChannel> {
        self.web_channel
            .as_ref()
            .map(|webchannel| ShowWebChannel::from(webchannel.name.as_str()))
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebChannel {
    pub name: String,
    #[serde(rename = "officialSite")]
    pub official_site: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    pub name: String,
    pub country: Country,
    #[serde(rename = "officialSite")]
    pub official_site_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
