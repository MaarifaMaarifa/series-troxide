use serde::{Deserialize, Serialize};

use super::{get_pretty_json_from_url, ApiError};

#[derive(PartialEq)]
pub enum ImageType {
    Poster,
    Banner,
    Background,
    Typography,
    Other,
}

impl From<&str> for ImageType {
    fn from(value: &str) -> Self {
        match value {
            "poster" => Self::Poster,
            "banner" => Self::Banner,
            "typography" => Self::Typography,
            "background" => Self::Background,
            _ => Self::Other,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Image {
    pub id: u32,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub resolutions: Resolutions,
}

impl Image {
    pub fn get_image_type(&self) -> Option<ImageType> {
        self.kind
            .as_ref()
            .map(|kind| ImageType::from(kind.as_str()))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Resolutions {
    pub original: OriginalResolution,
    pub medium: Option<MediumResolution>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OriginalResolution {
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MediumResolution {
    pub url: String,
}

// Relplace ID with the actual series id
const IMAGES_ADDRESS: &str = "https://api.tvmaze.com/shows/ID/images";

/// Retrieves all the images available for the given series id
pub async fn get_show_images(series_id: u32) -> Result<String, ApiError> {
    let url = IMAGES_ADDRESS.replace("ID", &series_id.to_string());

    get_pretty_json_from_url(url)
        .await
        .map_err(ApiError::Network)
}

// /// Loads the most recent image banner from the provided series id
// pub async fn get_recent_banner(series_id: u32) -> Option<Vec<u8>> {
//     let images = get_show_images(series_id).await.ok()?;
//     let recent_banner = images
//         .into_iter()
//         .filter(|image| ImageType::new(image) == ImageType::Banner)
//         .max_by_key(|image| image.id)?;
//     println!("done getting maximum");

//     load_image(recent_banner.resolutions.original.url).await
// }
