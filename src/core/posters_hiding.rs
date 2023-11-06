//! Prevent certain series posters from appearing in the Discover page

use std::collections::HashSet;
use std::path;

use indexmap::IndexMap;
use lazy_static::lazy_static;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::paths;

const HIDDEN_SERIES_FILENAME: &str = "hidden-series";

lazy_static! {
    pub static ref HIDDEN_SERIES: RwLock<HiddenSeries> = RwLock::new(HiddenSeries::new());
}

#[derive(Clone)]
pub struct HiddenSeries {
    /// <`Series ID`, (`Series Name`, `Premiere Date`)>
    hidden_series: Option<IndexMap<u32, (String, Option<String>)>>,
    hidden_series_filepath: path::PathBuf,
}

impl HiddenSeries {
    fn new() -> Self {
        let mut hidden_series_filepath = paths::PATHS
            .read()
            .expect("failed to read paths")
            .get_config_dir_path()
            .to_path_buf();

        hidden_series_filepath.push(HIDDEN_SERIES_FILENAME);

        Self {
            hidden_series: None,
            hidden_series_filepath,
        }
    }

    pub async fn load_series(&mut self) -> anyhow::Result<()> {
        match fs::read_to_string(&self.hidden_series_filepath).await {
            Ok(file_content) => {
                self.hidden_series = Some(serde_json::from_str(&file_content)?);
            }
            Err(err) => {
                if let std::io::ErrorKind::NotFound = err.kind() {
                    self.hidden_series = Some(IndexMap::new());
                } else {
                    anyhow::bail!(err)
                }
            }
        };
        Ok(())
    }

    pub async fn get_hidden_series_ids(&mut self) -> Option<HashSet<u32>> {
        self.load_series()
            .await
            .map_err(|err| warn!("could not load hidden posters: {}", err))
            .ok()?;

        self.get_hidden_series()
            .map(|hidden_series| hidden_series.keys().copied().collect::<HashSet<u32>>())
    }

    /// Unhides a Series and automatically save it to it's file
    pub async fn unhide_series(&mut self, series_id: u32) -> anyhow::Result<()> {
        if let Some(ref mut hidden_series) = self.hidden_series {
            hidden_series.shift_remove(&series_id);
            self.save_series().await?;
        }

        Ok(())
    }

    pub fn get_hidden_series(&self) -> Option<&IndexMap<u32, (String, Option<String>)>> {
        self.hidden_series.as_ref()
    }

    /// Hides a Series and automatically save it to it's file
    pub async fn hide_series(
        &mut self,
        series_id: u32,
        series_name: String,
        premier_date: Option<String>,
    ) -> anyhow::Result<()> {
        loop {
            if let Some(ref mut hidden_series) = self.hidden_series {
                hidden_series.insert(
                    series_id,
                    (
                        series_name.to_owned(),
                        premier_date.map(|date| date.to_owned()),
                    ),
                );
                break;
            } else {
                self.load_series().await?;
            }
        }

        self.save_series().await?;

        info!("'{}' successfully hidden'", series_name);

        Ok(())
    }

    pub async fn save_series(&self) -> anyhow::Result<()> {
        let file_content = if let Some(hidden_series) = &self.hidden_series {
            serde_json::to_string_pretty(&hidden_series)?
        } else {
            let map: IndexMap<u32, (String, Option<String>)> = IndexMap::new();
            serde_json::to_string_pretty(&map)?
        };

        fs::write(&self.hidden_series_filepath, file_content).await?;

        Ok(())
    }
}

impl Default for HiddenSeries {
    fn default() -> Self {
        Self::new()
    }
}
