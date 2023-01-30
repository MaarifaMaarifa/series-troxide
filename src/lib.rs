use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::RangeInclusive;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    season_number: u32,
    episodes: HashSet<Episode>,
}

impl Season {
    fn new(season_number: u32) -> Self {
        Self {
            season_number,
            episodes: HashSet::new(),
        }
    }
    /// Adds an episode into a season
    fn add_episode(&mut self, episode: Episode) -> Result<()> {
        if !self.episodes.insert(episode) {
            return Err(anyhow!(SeasonError::EpisodeExists(episode)));
        };
        Ok(())
    }

    /// Adds episodes using the provided range
    fn add_episode_range(&mut self, episode_range: RangeInclusive<u32>) {
        for episode in episode_range {
            if let Err(err) = self.add_episode(episode) {
                eprintln!("Warning: {}", err)
            };
        }
    }

    /// Removes an episode from a season
    fn remove_episode(&mut self, episode: Episode) -> Result<()> {
        if !self.episodes.remove(&episode) {
            return Err(anyhow!(SeasonError::EpisodeNotFound(episode)));
        };
        Ok(())
    }

    fn get_episodes_summary(&self) -> summary::EpisodesSummary {
        let episodes: Vec<Episode> = self.episodes.iter().copied().collect();
        summary::EpisodesSummary::new(episodes)
    }

    /// Get the total number of episodes in a season
    fn get_total_episodes(&self) -> usize {
        self.episodes.len()
    }

    /// Get all episodes from the season
    fn get_episodes(&self) -> Vec<Episode> {
        let episodes: Vec<Episode> = self.episodes.iter().copied().collect();
        episodes
    }
}

#[derive(Debug, Error)]
pub enum SeriesError {
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

        let season = Season::new(season_number);
        self.seasons.insert(season_number, season);

        Ok(())
    }

    /// Add new seasons using the specified season range
    pub fn add_season_range(&mut self, season_range: RangeInclusive<u32>) -> Result<()> {
        // Checking if any of season in range exist before adding them to the collection
        for season in season_range.clone() {
            if self.seasons.contains_key(&season) {
                return Err(anyhow!(SeriesError::SeasonAlreadyExists(season)));
            }
        }   

        // Now adding the season after conferming that they all don't exist
        for season_num in season_range {
            let season = Season::new(season_num);
            self.seasons.insert(season_num, season);
        }

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

    pub fn add_episode_range(&mut self, season_number: u32, episode_range: RangeInclusive<u32>) -> Result<()>{
        if let Some(season) = self.seasons.get_mut(&season_number) {
            season.add_episode_range(episode_range);
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

    /// Changes the episode duration of the series using the provided episode duration
    pub fn change_episode_duration(&mut self, episode_duration: u32) {
        self.episode_duration = episode_duration;
    }

    /// Get all episodes on the given season
    fn get_episodes(&self, season: u32) -> Result<Vec<Episode>, SeriesError> {
        if let Some(season) = self.seasons.get(&season) {
            return Ok(season.get_episodes());
        } 
        Err(SeriesError::SeasonNotFound(season))
    }

    pub fn get_episodes_summary(&self, season: u32) -> Result<summary::EpisodesSummary, SeriesError> {
        let episode_summary = summary::EpisodesSummary::new(self.get_episodes(season)?);
        Ok(episode_summary)
    }

    /// Get all the Season Summary from the series
    pub fn get_seasons_summary(&self) -> Option<Vec<summary::SeasonSummary>> {
        let seasons_number = self.get_total_seasons();

        if seasons_number == 0 {
            return None
        }

        let mut seasons_summaries = Vec::with_capacity(seasons_number);

        for season in self.seasons.values() {
            let season_summary = summary::SeasonSummary::new(season);
            seasons_summaries.push(season_summary);
        }

        // Getting a sorted season summary for clarity
        seasons_summaries.sort_by(|a, b| a.season.season_number.cmp(&b.season.season_number));

        Some(seasons_summaries)
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
    /// Unsorted (default option when unspecified)
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
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SeriesCollection {
    collection: Vec<Series>,
}

impl SeriesCollection {
    /// Creates a instance of the database collection from contents read from a 
    /// a database file
    pub fn load_series_with_db_content(database_content: &str) -> Result<Self> {
        let series_collection: Self = ron::from_str(database_content)?;
        Ok(series_collection)
    }

    /// Loads series from a ron file and returns Self
    pub fn load_series_with_db_path(path: &Path) -> Result<Self> {
        /* Attempts to read the file if it exists, when not it will create a new
        empty ron file, by creating it's directory first */
        let file_content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    fs::create_dir_all(
                        path.parent()
                            .context("Could not obtain the database directory")?,
                    )
                    .context("Could not create database directory")?;

                    // creating empty database content
                    // SAFETY: The unwrap here is guaranteed to never panic
                    let default_empty_database = ron::to_string(&SeriesCollection::default()).unwrap();

                    fs::write(path, &default_empty_database)
                        .context("Could not create database")?;
                    default_empty_database
                }
                err => return Err(anyhow!(err)),
            },
        };

        Self::load_series_with_db_content(&file_content)
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

    /// Change the name of a particular series by providing it's old name and new name
    pub fn change_series_name(&mut self, old_name: &str, new_name: String) -> Result<(), SeriesCollectionError> {
        let series = self.get_series_mut(old_name)?;
        series.name = new_name;
        Ok(())
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
    pub fn get_summary(&self, series_name: &str) -> Result<summary::SeriesSummary> {
        let summary = summary::SeriesSummary::new(self.get_series(series_name)?);
        Ok(summary)
    }

    /// Saves the series collection into the ron file
    pub fn save_file(&self, path: impl AsRef<Path>) -> Result<()> {
        // Choosing the config depth_limit of 4 because it provides good visual of the ron file
        // that might be usefull for manual inspection of the file
        let config = ron::ser::PrettyConfig::new().depth_limit(4);
        let file_contents = ron::ser::to_string_pretty(&self, config)?;
        fs::write(path, file_contents)?;
        Ok(())
    }
}


pub mod summary {
    use super::*;
    use std::fmt::Display;

    /// Summary for sorted episodes
    /// Provides sorted summary of episodes for the given Vec of Episodes
    pub struct EpisodesSummary {
        episodes: Vec<Episode>,
    }

    impl EpisodesSummary {
        pub fn new(mut episodes: Vec<Episode>) -> Self {
            episodes.sort();
            Self {episodes}
        }
    }

    impl Display for EpisodesSummary {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut summary = String::new();

            for episode in &self.episodes {
                summary.push_str(
                    &format!("{} ", episode)
                )
            }

            write!(f, "{}", summary)
        }
    }

    /// Summary for a Season
    /// Provide useful summary information for a particular season
    pub struct SeasonSummary<'a> {
        pub season: &'a Season,
    }

    impl<'a> SeasonSummary<'a> {
        pub fn new(season: &'a Season) -> Self {
            Self {season}
        }
    }

    impl<'a> Display for SeasonSummary<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Season: {}\nEpisodes: {}",
                self.season.season_number,
                self.season.get_episodes_summary())
        }
    }

    
    /// Summary for a Series
    /// Provide useful summary information for a paricular series
    pub struct SeriesSummary<'a> {
        series: &'a Series,
    }

    impl<'a> SeriesSummary<'a> {
        /// Creates a new instance of series summary using the supplied &Series
        pub fn new(series: &'a Series) -> Self {
            Self {series}
        }
    }

    impl<'a> Display for SeriesSummary<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut season_episodes: Vec<(_, _)> = self.series
                .seasons
                .iter()
                .map(|(season, episode)| (season, episode.get_total_episodes()))
                .collect();

            season_episodes.sort_by_key(|x| x.0);

            let mut summary = format!(
                "\
Series Name: {}
Episode Duration: {} mins
Total Seasons: {}
Total Episodes: {}",
                self.series.name,
                self.series.episode_duration,
                self.series.get_total_seasons(),
                self.series.get_total_episodes(),
            );

            // Appending the {season} => {episode} information to the summary
            for (season, episode) in season_episodes {
                summary.push_str(&format!("\nSeason {} => {} Episodes", season, episode));
            }

            write!(f, "{}", summary)
        }
    }
}
