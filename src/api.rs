use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Rating {
    average: f32,
}

pub mod series_searching {
    use super::*;

    // The series name goes after the equals sign
    const SERIES_SEARCH_ADDRESS: &str = "https://api.tvmaze.com/search/shows?q=";

    #[derive(Debug, Deserialize)]
    pub struct SeriesSearchResult {
        pub show: Show,
    }

    #[derive(Debug, Deserialize)]
    pub struct Show {
        pub id: u32,
        pub name: String,
        pub premiered: Option<String>,
        pub genres: Vec<String>,
    }

    pub fn search_series(series_name: &str) -> Result<Vec<SeriesSearchResult>, reqwest::Error> {
        let url = format!("{}{}", SERIES_SEARCH_ADDRESS, series_name);
        reqwest::blocking::get(url)?.json()
    }
}

pub mod series_information {
    use super::*;

    // The series id goes after the last slash(append at the end of the string)
    const SERIES_INFORMATION_ADDRESS: &str = "https://api.tvmaze.com/shows/";

    #[derive(Debug, Deserialize)]
    pub struct SeriesMainInformation {
        name: String,
        language: String,
        genres: Vec<String>,
        status: String,
        #[serde(rename = "averageRuntime")]
        average_runtime: u32,
        premiered: Option<String>,
        ended: Option<String>,
        rating: Rating,
        network: Option<String>,
        #[serde(rename = "webChannel")]
        web_channel: WebChannel,
        summary: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct WebChannel {
        name: String,
        #[serde(rename = "officialSite")]
        official_site: String,
    }

    pub fn get_series_main_info(series_id: u32) -> Result<SeriesMainInformation, reqwest::Error> {
        let url = format!("{}{}", SERIES_INFORMATION_ADDRESS, series_id);
        reqwest::blocking::get(url)?.json()
    }
}

pub mod seasons_list {
    use super::*;

    // replace the word SERIES-ID with the actual series id
    const SEASONS_LIST_ADDRESS: &str = "https://api.tvmaze.com/shows/SERIES-ID/seasons";

    #[derive(Debug, Deserialize)]
    pub struct Season {
        number: u32,
        #[serde(rename = "episodeOrder")]
        episode_order: u32,
        #[serde(rename = "premiereDate")]
        premiere_date: Option<String>,
        #[serde(rename = "endDate")]
        end_date: Option<String>,
    }

    pub fn get_seasons_list(series_id: u32) -> Result<Vec<Season>, reqwest::Error> {
        let url = SEASONS_LIST_ADDRESS.replace("SERIES-ID", &series_id.to_string());
        reqwest::blocking::get(url)?.json()
    }
}

pub mod episodes_information {
    use super::*;

    const EPISODE_INFORMATION_ADDRESS: &str =
        "https://api.tvmaze.com/shows/SERIES-ID/episodebynumber?season=SEASON&number=EPISODE";

    #[derive(Debug, Deserialize)]
    pub struct Episode {
        name: String,
        season: u32,
        number: u32,
        runtime: u32,
        airdate: Option<String>,
        airtime: String, // can be empty
        airstamp: String,
        rating: Rating,
        image: Image,
        summary: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Image {
        #[serde(rename = "original")]
        original_image_url: String,
        #[serde(rename = "medium")]
        medium_image_url: String,
    }

    pub fn get_episode_information(
        series_id: u32,
        season: u32,
        episode: u32,
    ) -> Result<Episode, reqwest::Error> {
        let url = EPISODE_INFORMATION_ADDRESS.replace("SERIES-ID", &series_id.to_string());
        let url = url.replace("SEASON", &season.to_string());
        let url = url.replace("EPISODE", &episode.to_string());

        reqwest::blocking::get(url)?.json()
    }
}
