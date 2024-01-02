use serde::Deserialize;

pub use super::AgeError;
use crate::core::api::tv_maze::{get_pretty_json_from_url, ApiError};

#[derive(Deserialize, Debug, Clone)]
pub struct Crew {
    #[serde(rename = "type")]
    pub kind: String,
    pub person: super::Person,
}

// replace ID with the actual show id
const SHOW_CREW_ADDRESS: &str = "https://api.tvmaze.com/shows/ID/crew";

pub async fn get_show_crew(series_id: u32) -> Result<String, ApiError> {
    let url = SHOW_CREW_ADDRESS.replace("ID", &series_id.to_string());

    get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)
}
