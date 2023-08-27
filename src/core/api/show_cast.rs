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
    pub gender: Option<String>,
    pub birthday: Option<String>,
    pub deathday: Option<String>,
    pub country: Option<Country>,
    pub image: Option<Image>,
}

#[derive(Debug, thiserror::Error)]
pub enum AgeError {
    #[error("no birthdate found in cast information")]
    NotFound,

    #[error("failed to parse the birthdate")]
    Parse(chrono::ParseError),
}

impl Cast {
    pub fn duration_since_birth(&self) -> Result<chrono::Duration, AgeError> {
        let date = self.person.birthday.as_ref().ok_or(AgeError::NotFound)?;
        let birthdate =
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(AgeError::Parse)?;

        let current_date = chrono::Local::now().date_naive();

        Ok(current_date.signed_duration_since(birthdate))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Country {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Character {
    pub name: String,
    pub image: Option<Image>,
}

// replace ID with the actual show id
const SHOW_CAST_ADDRESS: &str = "https://api.tvmaze.com/shows/ID/cast";

pub async fn get_show_cast(series_id: u32) -> Result<String, ApiError> {
    let url = SHOW_CAST_ADDRESS.replace("ID", &series_id.to_string());

    get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)
}
