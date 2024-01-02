use std::io::ErrorKind;

use tracing::info;

use super::{CacheFilePath, CACHER};
use crate::core::api::tv_maze::deserialize_json;
use crate::core::api::tv_maze::people::show_cast::{self, Cast};
use crate::core::api::tv_maze::people::show_crew::{self, Crew};
use crate::core::api::tv_maze::ApiError;
use crate::core::caching::{read_cache, write_cache};

pub async fn get_show_cast(series_id: u32) -> Result<Vec<Cast>, ApiError> {
    let series_cast_filepath = CACHER.get_cache_file_path(CacheFilePath::SeriesShowCast(series_id));

    let json_string = match read_cache(&series_cast_filepath).await {
        Ok(json_string) => json_string,
        Err(err) => {
            info!("falling back online for 'show cast' for series id: {series_id}");
            let json_string = show_cast::get_show_cast(series_id).await?;
            if err.kind() == ErrorKind::NotFound {
                write_cache(&json_string, &series_cast_filepath).await;
            }
            json_string
        }
    };
    deserialize_json(&json_string)
}

pub async fn get_show_crew(series_id: u32) -> Result<Vec<Crew>, ApiError> {
    let series_crew_filepath = CACHER.get_cache_file_path(CacheFilePath::SeriesShowCrew(series_id));

    let json_string = match read_cache(&series_crew_filepath).await {
        Ok(json_string) => json_string,
        Err(err) => {
            info!("falling back online for 'show crew' for series id: {series_id}");
            let json_string = show_crew::get_show_crew(series_id).await?;
            if err.kind() == ErrorKind::NotFound {
                write_cache(&json_string, &series_crew_filepath).await;
            }
            json_string
        }
    };
    deserialize_json(&json_string)
}
