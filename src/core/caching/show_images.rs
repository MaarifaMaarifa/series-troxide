use std::io::ErrorKind;

use super::{
    load_image, read_cache, write_cache, CacheFilePath, ImageKind, ImageResolution, CACHER,
};
use crate::core::api::tv_maze::{
    deserialize_json,
    show_images::{get_show_images as get_show_images_api, Image, ImageType},
    ApiError,
};
use tracing::info;

pub async fn get_show_images(series_id: u32) -> Result<Vec<Image>, ApiError> {
    let series_image_list_path =
        CACHER.get_cache_file_path(CacheFilePath::SeriesImageList(series_id));

    let image_list_json = match read_cache(&series_image_list_path).await {
        Ok(info) => info,
        Err(err) => {
            info!(
                "falling back online for 'series image list' for series id {}",
                series_id
            );
            let json_string = get_show_images_api(series_id).await?;
            if err.kind() == ErrorKind::NotFound {
                write_cache(&json_string, &series_image_list_path).await;
            }
            json_string
        }
    };

    deserialize_json(&image_list_json)
}

/// Loads the most recent image banner from the provided series id
pub async fn get_recent_banner(series_id: u32) -> Option<bytes::Bytes> {
    let images = get_show_images(series_id).await.ok()?;

    // Trying to take a background first if any
    if let Some(recent_background) = images
        .iter()
        .filter(|image| image.get_image_type() == Some(ImageType::Background))
        .last()
    {
        return load_image(
            recent_background.resolutions.original.url.clone(),
            ImageResolution::Original(ImageKind::Background),
        )
        .await;
    };

    // Falling to anything that is not a poster as poster dimensions don't look great as a background
    let recent_banner = images
        .into_iter()
        .filter(|image| image.get_image_type() != Some(ImageType::Poster))
        .last()?;

    load_image(
        recent_banner.resolutions.original.url,
        ImageResolution::Original(ImageKind::Background),
    )
    .await
}
