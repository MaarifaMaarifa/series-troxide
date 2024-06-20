use crate::core::api::tv_maze::Image;
use serde::Deserialize;

pub mod show_cast;
pub mod show_crew;

#[derive(Deserialize, Debug, Clone)]
pub struct Country {
    pub name: String,
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
    BirthdateNotFound,

    #[error("no deathdate found in cast information")]
    DeathdateNotFound,

    #[error("failed to parse the birthdate")]
    Parse(chrono::ParseError),
}

impl Person {
    pub fn birth_naive_date(&self) -> Result<chrono::NaiveDate, AgeError> {
        let date = self.birthday.as_ref().ok_or(AgeError::BirthdateNotFound)?;

        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(AgeError::Parse)
    }

    pub fn death_naive_date(&self) -> Result<chrono::NaiveDate, AgeError> {
        let date = self.deathday.as_ref().ok_or(AgeError::DeathdateNotFound)?;

        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(AgeError::Parse)
    }

    pub fn duration_since_birth(&self) -> Result<chrono::Duration, AgeError> {
        let birthdate = self.birth_naive_date()?;

        let current_date = chrono::Local::now().date_naive();

        Ok(current_date.signed_duration_since(birthdate))
    }

    pub fn age_duration_before_death(&self) -> Result<chrono::Duration, AgeError> {
        let birthdate = self.birth_naive_date()?;
        let deathdate = self.death_naive_date()?;

        Ok(deathdate.signed_duration_since(birthdate))
    }
}
