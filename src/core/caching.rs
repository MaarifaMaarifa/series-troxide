use std::path;

use crate::core::api::{self, deserialize_json};

use super::api::{series_information::SeriesMainInformation, ApiError};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use tokio::fs;
use tracing::info;

const SERIES_CACHE_DIRECTORY: &str = "series-troxide-series-data";

lazy_static! {
    pub static ref CACHER: Cacher = Cacher::init();
}

pub struct Cacher {
    cache_path: path::PathBuf,
}

impl Cacher {
    pub fn init() -> Self {
        info!("opening cache");
        if let Some(proj_dir) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
            let mut cache_path = path::PathBuf::from(&proj_dir.data_dir());
            cache_path.push(SERIES_CACHE_DIRECTORY);

            return Self { cache_path };
        } else {
            panic!("could not get the cache path");
        }
    }

    pub fn get_cache_path(&self) -> path::PathBuf {
        self.cache_path.clone()
    }
}

pub mod series_information {
    use super::api::series_information;
    use super::*;

    pub async fn get_series_main_info_with_url(
        url: String,
    ) -> Result<SeriesMainInformation, ApiError> {
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
        let name = format!("{}", series_id);
        let mut series_main_info_path = CACHER.get_cache_path();
        series_main_info_path.push(&name); // creating the series folder path
        let series_directory = series_main_info_path.clone(); // creating a copy before we make it path to file
        series_main_info_path.push(&name); // creating the series information json filename path

        let series_information_json = match fs::read_to_string(&series_main_info_path).await {
            Ok(info) => info,
            Err(err) => {
                info!(
                    "falling back online for 'series information' with id {}: {}",
                    series_id, err
                );
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(series_directory)
                    .await
                    .unwrap();
                let (series_information, json_string) =
                    series_information::get_series_main_info_with_id(series_id)
                        .await?
                        .get_data();
                fs::write(series_main_info_path, json_string).await.unwrap();
                return Ok(series_information);
            }
        };

        deserialize_json(&series_information_json)
    }

    pub async fn get_series_main_info_with_ids(
        series_ids: Vec<String>,
    ) -> Vec<SeriesMainInformation> {
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
}
