use serde::Deserialize;

use super::{get_pretty_json_from_url, ApiError, Image};

#[derive(Deserialize, Debug, Clone)]
pub struct Cast {
    pub person: Person,
    pub character: Character,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Person {
    pub name: String,
    pub image: Option<Image>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Character {
    pub name: String,
}

// replace ID with the actual show id
const SHOW_CAST_ADDRESS: &str = "https://api.tvmaze.com/shows/ID/cast";

pub async fn get_show_cast(series_id: u32) -> Result<String, ApiError> {
    let url = SHOW_CAST_ADDRESS.replace("ID", &series_id.to_string());

    get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)
}
