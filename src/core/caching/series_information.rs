use super::api::series_information;
use super::*;

use std::io::ErrorKind;

pub async fn get_series_main_info_with_url(url: String) -> Result<SeriesMainInformation, ApiError> {
    let id = url
        .split('/')
        .last()
        .expect("invalid url, no series id at the end of url")
        .parse::<u32>()
        .expect("could not parse series id from url");

    get_series_main_info_with_id(id).await
}

pub async fn get_series_main_info_with_id(
    series_id: u32,
) -> Result<SeriesMainInformation, ApiError> {
    let series_information_path =
        CACHER.get_cache_file_path(CacheFilePath::SeriesMainInformation(series_id));

    let series_information_json = match read_cache(&series_information_path).await {
        Ok(json_string) => json_string,
        Err(err) => {
            info!("falling back online for 'series information' for series id: {series_id}");
            let json_string = series_information::get_series_main_info_with_id(series_id).await?;

            if err.kind() == ErrorKind::NotFound {
                write_cache(&json_string, &series_information_path).await;
            }
            json_string
        }
    };
    deserialize_json(&series_information_json)
}

pub async fn get_series_main_info_with_ids(series_ids: Vec<String>) -> Vec<SeriesMainInformation> {
    let handles: Vec<_> = series_ids
        .iter()
        .map(|id| tokio::spawn(get_series_main_info_with_id(id.parse().unwrap())))
        .collect();

    let mut series_infos = Vec::with_capacity(handles.len());
    for handle in handles {
        let series_info = handle.await.unwrap().unwrap();
        series_infos.push(series_info);
    }
    series_infos
}
