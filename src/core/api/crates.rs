//! crates.io API

use serde::Deserialize;
use thiserror::Error;

/// The crate name goes at the end of the url
const CRATE_INFO_URL: &str = "https://crates.io/api/v1/crates/";

#[derive(Debug, Clone, Deserialize)]
pub struct CrateInformation {
    #[serde(rename = "crate")]
    pub package: Crate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Crate {
    newest_version: String,
    updated_at: String,
}

impl Crate {
    pub fn newest_version(&self) -> &str {
        &self.newest_version
    }

    pub fn is_up_to_date(&self) -> bool {
        let crates_io_version =
            semver::Version::parse(&self.newest_version).expect("parse version");
        let program_version =
            semver::Version::parse(env!("CARGO_PKG_VERSION")).expect("parse version");
        program_version >= crates_io_version
    }

    pub fn updated_at(&self) -> Result<chrono::NaiveDate, chrono::ParseError> {
        Ok(chrono::DateTime::parse_from_rfc3339(&self.updated_at)?.date_naive())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(reqwest::Error),
    #[error("deserializtion error: {0}")]
    Deserialization(serde_json::Error),
}

/// Requests information of `Series Troxide` from crates.io
pub async fn get_program_info() -> Result<CrateInformation, reqwest::Error> {
    let url = format!("{}{}", CRATE_INFO_URL, env!("CARGO_PKG_NAME"));

    let client = reqwest::Client::new();

    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .send()
        .await?;

    response.json().await
}
