use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time;
use thiserror::Error;

#[derive(Debug, Error)]
enum SeasonError {
    #[error("episode '{0}' does not exist")]
    EpisodeNotFound(u32),

    #[error("episode '{0}' already exists")]
    EpisodeExists(u32),
}

type Episode = u32;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Season {
    episodes: HashSet<Episode>,
}

impl Season {
    /// Adds an episode into a season
    fn add_episode(&mut self, episode: Episode) -> Result<()> {
        if !self.episodes.insert(episode) {
            return Err(anyhow!(SeasonError::EpisodeExists(episode)));
        };
        Ok(())
    }

    /// Removes an episode from a season
    fn remove_episode(&mut self, episode: Episode) -> Result<()> {
        if !self.episodes.remove(&episode) {
            return Err(anyhow!(SeasonError::EpisodeNotFound(episode)));
        };
        Ok(())
    }

    /// Get the total number of episodes in a season
    fn get_total_episodes(&self) -> usize {
        self.episodes.len()
    }
}

#[derive(Debug, Error)]
enum SeriesError {
    #[error("season '{0}' does not exist")]
    SeasonNotFound(u32),

    #[error("season '{0}' already exists")]
    SeasonAlreadyExists(u32),
    // #[error("no available seasons currently assigned")]
    // EmptySeasons,
}

/// Struct Representing a Watched Series with it's name, episode
/// duration and it's seasons
#[derive(Debug, Serialize, Deserialize)]
pub struct Series {
    name: String,
    episode_duration: u32,         // Episode duration in minutes
    seasons: HashMap<u32, Season>, // hashmap for series number and series pair
}

impl Series {
    /// Creates a new instance of Series, initialized with it's
    /// name and episode duration
    pub fn new(name: String, episode_duration: u32) -> Self {
        Self {
            name,
            episode_duration,
            seasons: HashMap::new(),
        }
    }

    /// Adds a new season into the series
    pub fn add_season(&mut self, season_number: u32) -> Result<()> {
        if self.seasons.contains_key(&season_number) {
            return Err(anyhow!(SeriesError::SeasonAlreadyExists(season_number)));
        }

        let season = Season::default();
        self.seasons.insert(season_number, season);

        Ok(())
    }

    /// Removes the given season from the series instance, returning an error if the
    /// season does not exist
    pub fn remove_season(&mut self, season_number: u32) -> Result<()> {
        if self.seasons.remove(&season_number).is_none() {
            return Err(anyhow!(SeriesError::SeasonNotFound(season_number)));
        }

        Ok(())
    }

    /// Adds an episode on the given season
    ///
    /// # Errors
    /// This function returns an error when the season is not found
    ///  or when no seasons are assigned for the Series
    pub fn add_episode(&mut self, season_number: u32, episode_number: u32) -> Result<()> {
        if let Some(season) = self.seasons.get_mut(&season_number) {
            season.add_episode(episode_number)?;
        } else {
            return Err(anyhow!(SeriesError::SeasonNotFound(season_number)));
        }

        Ok(())
    }

    /// Removes an episode on the given season
    pub fn remove_episode(&mut self, season_number: u32, episode_number: u32) -> Result<()> {
        if let Some(season) = self.seasons.get_mut(&season_number) {
            season.remove_episode(episode_number)?;
        } else {
            return Err(anyhow!(SeriesError::SeasonNotFound(season_number)));
        }

        Ok(())
    }

    /// Get total episodes in the series
    pub fn get_total_episodes(&self) -> usize {
        self.seasons
            .values()
            .map(|season| season.get_total_episodes())
            .sum()
    }

    /// Get total watch time in the series
    pub fn get_total_watch_time(&self) -> time::Duration {
        time::Duration::from_secs(
            self.get_total_episodes() as u64 * self.episode_duration as u64 * 60,
        )
    }

    /// Get total seasons in the series
    pub fn get_total_seasons(&self) -> usize {
        self.seasons.len()
    }
}

impl PartialEq for Series {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Enum providing different ways to sort series in the collection
#[derive(clap::Subcommand)]
pub enum SeriesSort {
    /// Unsorted
    Default,

    /// Sort based on watch time
    WatchTime,

    /// Sort based on watch time but reversed
    WatchTimeRev,

    /// Sort based on episode count
    EpisodeCount,

    /// Sort based on episode count but reversed
    EpisodeCountRev,

    /// Sort based on season count
    SeasonCount,

    /// Sort based on season count but reversed
    SeasonCountRev,
}

#[derive(Debug, Error)]
pub enum SeriesCollectionError {
    #[error("series '{0}' already exists")]
    SeriesAlreadyExists(String),

    #[error("series '{0}' does not exist")]
    SeriesNotFound(String),
}

/// struct providing the abstraction for the whole series currently tracked
#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesCollection {
    collection: Vec<Series>,
}

impl SeriesCollection {
    /// Loads series from a ron file and returns Self
    pub fn load_series(path: impl AsRef<Path>) -> Result<Self> {
        let file_content = fs::read_to_string(path)?;

        let series_collection: Self = ron::from_str(&file_content)?;

        Ok(series_collection)
    }

    /// Adds a new series to the collection
    pub fn add_series(&mut self, name: String, episode_duration: u32) -> Result<()> {
        let series = Series::new(name, episode_duration);
        if self.collection.contains(&series) {
            return Err(anyhow!(SeriesCollectionError::SeriesAlreadyExists(
                series.name
            )));
        }
        self.collection.push(series);

        Ok(())
    }

    /// Removes series from series collection
    pub fn remove_series(&mut self, series_name: &str) -> Result<(), SeriesCollectionError> {
        if let Some(series_index) = self
            .collection
            .iter()
            .position(|series| series.name == series_name)
        {
            self.collection.swap_remove(series_index);
            return Ok(());
        }
        Err(SeriesCollectionError::SeriesNotFound(
            series_name.to_string(),
        ))
    }

    /// Get an immutable reference from the series collection
    pub fn get_series(&self, series_name: &str) -> Result<&Series, SeriesCollectionError> {
        for series in &self.collection {
            if series.name == series_name {
                return Ok(series);
            }
        }
        Err(SeriesCollectionError::SeriesNotFound(
            series_name.to_string(),
        ))
    }

    /// Get a mutable reference from the series collection
    pub fn get_series_mut(
        &mut self,
        series_name: &str,
    ) -> Result<&mut Series, SeriesCollectionError> {
        for series in &mut self.collection {
            if series.name == series_name {
                return Ok(series);
            }
        }
        Err(SeriesCollectionError::SeriesNotFound(
            series_name.to_string(),
        ))
    }

    /// Get names of series based on different sorting
    pub fn get_series_names_sorted(&mut self, sort: SeriesSort) -> Vec<&String> {
        match sort {
            SeriesSort::Default => { /* Do nothing as no sort specified */ }
            SeriesSort::WatchTime => {
                self.collection.sort_by_key(|a| a.get_total_watch_time());
            }
            SeriesSort::WatchTimeRev => {
                self.collection
                    .sort_by_key(|a| std::cmp::Reverse(a.get_total_watch_time()));
            }
            SeriesSort::EpisodeCount => {
                self.collection.sort_by_key(|a| a.get_total_episodes());
            }
            SeriesSort::EpisodeCountRev => {
                self.collection
                    .sort_by_key(|a| std::cmp::Reverse(a.get_total_episodes()));
            }
            SeriesSort::SeasonCount => {
                self.collection.sort_by_key(|a| a.get_total_seasons());
            }
            SeriesSort::SeasonCountRev => {
                self.collection
                    .sort_by_key(|a| std::cmp::Reverse(a.get_total_seasons()));
            }
        }
        self.collection.iter().map(|series| &series.name).collect()
    }

    /// Get the total watch time of the whole series
    pub fn get_total_watch_time(&self) -> time::Duration {
        self.collection
            .iter()
            .map(|series| series.get_total_watch_time())
            .sum()
    }

    /// Get summary of the given series name
    pub fn get_summary(&self, series_name: &str) -> Result<String> {
        let series = self.get_series(series_name)?;
        let mut season_episodes: Vec<(_, _)> = series .seasons .iter() .map(|(season, episode)| (season, episode.get_total_episodes())) .collect();

        season_episodes.sort_by_key(|x| x.0);

        let mut summary = format!(
            "\
Series Name: {}
Episode Duration: {} mins
Total Seasons: {}
Total Episodes: {}",
            series_name,
            series.episode_duration,
            series.get_total_seasons(),
            series.get_total_episodes(),
        );

        // Appending the {season} => {episode} information to the summary
        for (season, episode) in season_episodes {
            summary.push_str(&format!("\nSeason {} => {} Episodes", season, episode));
        }

        Ok(summary)
    }

    /// Saves the series collection into the ron file
    pub fn save_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let config = ron::ser::PrettyConfig::new().depth_limit(4);
        let file_contents = ron::ser::to_string_pretty(&self, config)?;
        fs::write(path, file_contents)?;
        Ok(())
    }
}
