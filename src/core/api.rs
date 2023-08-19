use bytes::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod episodes_information;
pub mod seasons_list;
pub mod series_information;
pub mod series_searching;
pub mod show_cast;
pub mod show_images;
pub mod tv_schedule;
pub mod updates;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("network error during request")]
    Network(reqwest::Error),
    #[error("tvmaze api error when deserializing json: unexpected '{0}'")]
    Deserialization(String, serde_json::Error),
    #[error("errored json from tvmaze: name: '{0}', message: '{1}'")]
    BadJson(String, String),
}

#[derive(Debug, Deserialize, Clone)]
struct BadResponse {
    name: String,
    message: String,
    // code: u32,
    // status: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rating {
    pub average: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    #[serde(rename = "original")]
    pub original_image_url: String,
    #[serde(rename = "medium")]
    pub medium_image_url: String,
}

/// Loads the image from the provided url
pub async fn lload_image(image_url: String) -> Option<Bytes> {
    loop {
        match reqwest::get(&image_url).await {
            Ok(response) => {
                if let Ok(bytes) = response.bytes().await {
                    break Some(bytes);
                }
            }
            Err(ref err) => {
                if err.is_request() {
                    random_async_sleep().await;
                } else {
                    break None;
                }
            }
        }
    }
}

/// T
fn try_bad_json(json_string: &str) -> Option<(String, String)> {
    if let Ok(bad_response) = serde_json::from_str::<BadResponse>(json_string) {
        Some((bad_response.name, bad_response.message))
    } else {
        None
    }
}

pub fn deserialize_json<'a, T: serde::Deserialize<'a>>(
    prettified_json: &'a str,
) -> Result<T, ApiError> {
    serde_json::from_str::<T>(prettified_json).map_err(|err| {
        if let Some(data) = try_bad_json(prettified_json) {
            return ApiError::BadJson(data.0, data.1);
        }

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
    let response = loop {
        match reqwest::get(&url).await {
            Ok(response) => break response,
            Err(err) => {
                if err.is_request() {
                    random_async_sleep().await;
                } else {
                    return Err(err);
                }
            }
        }
    };

    let text = response.text().await?;

    Ok(json::stringify_pretty(json::parse(&text).unwrap(), 1))
}

/// Sleeps the current thread asynchronously between 0-0.2 seconds choosing a random
/// value in between.
async fn random_async_sleep() {
    let random_val = rand::random::<u64>() / 100_000_000_000_000_000;
    tokio::time::sleep(std::time::Duration::from_millis(random_val)).await;
}
