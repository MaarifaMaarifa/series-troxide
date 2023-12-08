use std::io::Write;

use bytes::Bytes;
use tracing::error;

const POSTER_WIDTH: u32 = 480;
const POSTER_HEIGHT: u32 = 853;
const BACKGROUND_WIDTH: u32 = 1280;
const BACKGROUND_HEIGHT: u32 = 720;

pub enum ImageResolution {
    Original(ImageKind),
    Medium,
}

#[derive(Clone, Copy)]
pub enum ImageKind {
    Poster,
    Background,
}

/// Loads the image from the provided url
///
/// Since Original images from TvMaze may have extremely high resolution up to 4k which can cause `wgpu` to crash,
/// this function will thumbnail the original image to the size that is good enough to be displayed in the GUI.
pub async fn load_image(image_url: String, image_resolution: ImageResolution) -> Option<Bytes> {
    loop {
        match reqwest::get(&image_url).await {
            Ok(response) => {
                if let Ok(bytes) = response.bytes().await {
                    let image = image::load_from_memory(&bytes)
                        .map_err(|err| error!("failed to load image from the api: {}", err))
                        .ok()?;

                    break match image_resolution {
                        ImageResolution::Original(image_kind) => {
                            if should_lower_resolution(&image, image_kind) {
                                lower_image_resolution(image, image_kind)
                            } else {
                                Some(bytes)
                            }
                        }
                        ImageResolution::Medium => Some(bytes),
                    };
                }
            }
            Err(ref err) => {
                if err.is_request() {
                    super::random_async_sleep().await;
                } else {
                    break None;
                }
            }
        }
    }
}

fn should_lower_resolution(image: &image::DynamicImage, image_kind: ImageKind) -> bool {
    match image_kind {
        ImageKind::Poster => image.height() > POSTER_HEIGHT || image.width() > POSTER_WIDTH,
        ImageKind::Background => {
            image.height() > BACKGROUND_HEIGHT || image.width() > BACKGROUND_WIDTH
        }
    }
}

fn lower_image_resolution(
    image: image::DynamicImage,
    image_kind: ImageKind,
) -> Option<bytes::Bytes> {
    let img = match image_kind {
        ImageKind::Poster => image.thumbnail(POSTER_WIDTH, POSTER_WIDTH),
        ImageKind::Background => image.thumbnail(BACKGROUND_WIDTH, BACKGROUND_HEIGHT),
    };

    let mut writer = std::io::BufWriter::new(vec![]);

    let mut jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 100);

    jpeg_encoder
        .encode_image(&img)
        .map_err(|err| error!("failed to encode image: {}", err))
        .ok()?;

    writer
        .flush()
        .map_err(|err| error!("failed to flush image bytes: {}", err))
        .ok()?;

    Some(bytes::Bytes::copy_from_slice(writer.get_ref()))
}
