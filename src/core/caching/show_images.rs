use crate::core::{
    api::{
        deserialize_json,
        show_images::{get_show_images as get_show_images_api, Image, ImageType},
        ApiError,
    },
    caching::{load_image, CacheFolderType, CACHER},
};
use tokio::fs;
use tracing::info;

pub async fn get_show_images(series_id: u32) -> Result<Vec<Image>, ApiError> {
    let name = format!("{}", series_id);
    let mut series_images_list_path = CACHER.get_cache_folder_path(CacheFolderType::Series);
    series_images_list_path.push(&name); // creating the series folder path
    let series_directory = series_images_list_path.clone(); // creating a copy before we make it path to file
    series_images_list_path.push("series-images-list"); // creating the episode list json filename path

    let series_information_json = match fs::read_to_string(&series_images_list_path).await {
        Ok(info) => info,
        Err(err) => {
            info!(
                "falling back online for 'series images list' for series id {}: {}",
                series_id, err
            );
            fs::DirBuilder::new()
                .recursive(true)
                .create(series_directory)
                .await
                .unwrap();
            let json_string = get_show_images_api(series_id).await?;
            fs::write(series_images_list_path, &json_string)
                .await
                .unwrap();
            json_string
        }
    };

    deserialize_json(&series_information_json)
}

/// Loads the most recent image banner from the provided series id
pub async fn get_recent_banner(series_id: u32) -> Option<Vec<u8>> {
    let images = get_show_images(series_id).await.ok()?;
    let recent_banner = images
        .into_iter()
        .filter(|image| ImageType::new(image) != ImageType::Poster)
        .last()?;
    // .find(|image| {
    //     let image_type = ImageType::new(image);
    //     image_type == ImageType::Banner || image_type == ImageType::Background
    // })?;
    // .max_by_key(|image| image.id)?;
    println!("done getting maximum");

    load_image(recent_banner.resolutions.original.url).await

    // if (image_bytes.len() / 1_048_576) < 1 {
    //     Some(image_bytes)
    // } else {
    //     warn!("image exceeded 1MB for series id {series_id}");
    //     None
    // }
}
