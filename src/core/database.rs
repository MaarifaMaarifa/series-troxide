use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};
use tracing::info;

use super::{api::tv_maze::series_information::SeriesMainInformation, caching};
use crate::core::paths;

// The last digit represents the version of the database.
const DATABASE_FOLDER_NAME: &str = "series-troxide-db-1";

lazy_static! {
    pub static ref DB: Database = Database::init();
}

pub struct Database {
    db: Db,
}

impl Database {
    fn init() -> Self {
        let mut database_path = paths::PATHS
            .read()
            .expect("failed to read paths")
            .get_data_dir_path()
            .to_path_buf();

        info!("initializing database at {}", database_path.display());

        database_path.push(DATABASE_FOLDER_NAME);
        let db = sled::open(database_path).unwrap();
        if !db.was_recovered() {
            info!("created a fresh database as none was found");
        }
        Self { db }
    }

    /// Adds the given series to the database.
    ///
    /// # Note
    /// This will overwrite any previous series with the same id.
    pub fn add_series(&self, series_id: u32, series: &Series) {
        self.db
            .insert(series_id.to_string(), bincode::serialize(series).unwrap())
            .unwrap();
    }

    /// Removes a series in the database.
    ///
    /// # Note
    /// Does nothing when the series does not exist
    pub fn remove_series(&self, series_id: u32) {
        self.db.remove(series_id.to_string()).unwrap();
    }

    pub fn get_series(&self, series_id: u32) -> Option<Series> {
        let series_bytes = self.db.get(series_id.to_string()).unwrap()?;
        Some(bincode::deserialize(&series_bytes).unwrap())
    }

    pub fn get_series_collection(&self) -> Vec<Series> {
        self.db
            .iter()
            .values()
            .map(|series| {
                let series = series.unwrap();
                bincode::deserialize(&series).unwrap()
            })
            .collect()
    }

    pub fn get_series_id_collection(&self) -> Vec<String> {
        self.db
            .iter()
            .keys()
            .map(|series| {
                let series = series.unwrap();
                // bincode::deserialize(&series).unwrap()
                String::from_utf8_lossy(&series).into_owned()
            })
            .collect()
    }

    /// get series ids and their corresponding series structures
    pub fn get_ids_and_series(&self) -> Vec<(String, Series)> {
        self.db
            .iter()
            .map(|tup| {
                let (series_id, series) = tup.unwrap();
                let series_id = String::from_utf8_lossy(&series_id).into_owned();
                let series = bincode::deserialize::<Series>(&series).unwrap();
                (series_id, series)
            })
            .collect()
    }

    /// Returns the total number of series being tracked
    pub fn get_total_series(&self) -> usize {
        self.db.len()
    }

    /// Get the total amount of seasons watched across all
    /// series in the database
    pub fn get_total_seasons(&self) -> usize {
        self.get_series_collection()
            .iter()
            .map(|series| series.get_total_seasons())
            .sum()
    }

    /// Get the total amount of episodes watched across all
    /// series in the database
    pub fn get_total_episodes(&self) -> usize {
        self.get_series_collection()
            .iter()
            .map(|series| series.get_total_episodes())
            .sum()
    }

    pub fn export(&self) -> database_transfer::TransferData {
        database_transfer::TransferData::new(self.get_series_collection())
    }

    pub fn import(&self, transfer_data: &database_transfer::TransferData) {
        for series in transfer_data.get_series() {
            self.add_series(series.id, series);
        }
        self.db.flush().expect("flushing database");
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Series {
    id: u32,
    name: String,
    is_tracked: bool,
    seasons: HashMap<u32, Season>,
}

impl Series {
    /// Creates a new Series object
    ///
    /// # Note
    /// The series is initialized as Untracked, to mark the series as tracked
    /// self.mark_tracked() should be explicitly called.
    pub fn new(name: String, id: u32) -> Self {
        Self {
            id,
            name,
            is_tracked: false,
            seasons: HashMap::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Whether a series is being tracked or not
    ///
    /// Return True when is marked as tracked otherwise false
    pub fn is_tracked(&self) -> bool {
        self.is_tracked
    }

    /// Marks the series as being tracked
    pub fn mark_tracked(&mut self) {
        self.is_tracked = true;
    }

    /// Marks the series as not being tracked
    pub fn mark_untracked(&mut self) {
        self.is_tracked = false;
    }

    /// Updates the database with the current Series
    ///    
    /// This method exists  because Series object once created,
    /// has no connection to the database anymore and has to be rewritten to
    /// the database for the changes to be saved.
    ///
    /// # Note
    /// This method is automatically called when Series object goes out of scope
    /// as self.update() is called in it's drop implementation, hence no need of
    /// calling it unless if you want immediate update i.e. there is some code that
    /// would take time to run before the object is dropped.
    pub fn update(&self) {
        DB.add_series(self.id, self);
    }

    pub fn add_season(&mut self, season_number: u32) {
        self.seasons.insert(season_number, Season::new());
    }

    pub fn remove_season(&mut self, season_number: u32) {
        self.seasons.remove(&season_number);
    }

    /// adds an episode into the series
    ///
    /// returns a true if the episode is newly added into the series and vice versa is true
    ///
    /// # None
    /// tracks only when the supplied episode is watchable preventing allowing watched episodes that
    /// are released into the future.
    pub async fn add_episode(&mut self, season_number: u32, episode: Episode) -> bool {
        loop {
            if let Some(season) = self.seasons.get_mut(&season_number) {
                break season.track_episode(self.id, season_number, episode).await;
            } else {
                self.add_season(season_number);
            }
        }
    }

    /// adds an episode into the series
    ///
    /// returns a true if the episode is newly added into the series and vice versa is true
    ///
    /// # Note
    /// Does not check if the episode is watchable which is useful when importing episodes
    pub fn add_episode_unchecked(&mut self, season_number: u32, episode: Episode) {
        loop {
            if let Some(season) = self.seasons.get_mut(&season_number) {
                season.track_episode_unchecked(episode);
                break;
            } else {
                self.add_season(season_number);
            }
        }
    }

    pub async fn add_episodes(
        &mut self,
        season_number: u32,
        episodes_range: RangeInclusive<u32>,
    ) -> AddResult {
        loop {
            if let Some(season) = self.seasons.get_mut(&season_number) {
                break season
                    .track_episodes(self.id, season_number, episodes_range)
                    .await;
            } else {
                self.add_season(season_number);
            }
        }
    }

    /// removes an episode from the series
    pub fn remove_episode(&mut self, season_number: u32, episode_number: Episode) {
        if let Some(season) = self.seasons.get_mut(&season_number) {
            season.untrack_episode(episode_number)
        }
    }

    pub fn get_season(&self, season_number: u32) -> Option<&Season> {
        self.seasons.get(&season_number)
    }

    pub fn get_season_mut(&mut self, season_number: u32) -> Option<&mut Season> {
        self.seasons.get_mut(&season_number)
    }

    /// Get the total amount of seasons tracked
    pub fn get_total_seasons(&self) -> usize {
        self.seasons.len()
    }

    /// Returns total tracked episodes of the season
    pub fn get_total_episodes(&self) -> usize {
        self.seasons
            .values()
            .map(|season| season.get_total_episodes())
            .sum()
    }

    /// Return the last watched season together with it's number
    ///
    /// This obviously skip any unwatched season in between and just returns the highest
    pub fn get_last_season(&self) -> Option<(u32, &Season)> {
        self.seasons
            .iter()
            .filter(|(_, season)| season.get_total_episodes() != 0)
            .max_by(|x, y| x.0.cmp(y.0))
            .map(|(season_number, season)| (*season_number, season))
    }

    /// Get the total time that has been spent watching the series
    ///
    /// This method returns SeriesMainInformation associated with the Series
    /// together with it's total runtime
    pub async fn get_total_average_watchtime(&self) -> (SeriesMainInformation, Option<u32>) {
        let series_info = caching::series_information::get_series_main_info_with_id(self.id)
            .await
            .unwrap();
        let episode_average_watchtime = series_info.average_runtime;

        (
            series_info,
            episode_average_watchtime.map(|time| time * self.get_total_episodes() as u32),
        )
    }
}

impl Drop for Series {
    fn drop(&mut self) {
        // Making sure database series is updated
        self.update();

        // Preventing unwatched and untracked series from cloggin up the database.
        // This can happen when a user adds a series for tracking and untracks the
        // series without having any episodes checked.
        if !self.is_tracked() && self.get_total_episodes() == 0 {
            DB.remove_series(self.id);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season {
    episodes: HashSet<Episode>,
}

impl Season {
    pub fn new() -> Self {
        Self {
            episodes: HashSet::new(),
        }
    }

    /// adds the given episode to tracking
    ///
    /// tracks only when the supplied episode is watchable preventing allowing watched episodes that
    /// are released into the future.
    /// This method returns true if the episode was newly added and vice versa is true
    pub async fn track_episode(
        &mut self,
        series_id: u32,
        season_number: u32,
        episode_number: Episode,
    ) -> bool {
        let episode_list = caching::episode_list::EpisodeList::new(series_id)
            .await
            .expect("failed to get episode list");

        if let Some(episode) = episode_list.get_episode(season_number, episode_number) {
            if let Ok(false) = episode.is_future_release() {
                return self.episodes.insert(episode_number);
            }
        }
        false
    }

    /// adds the given episode to tracking
    ///
    /// # Note
    /// Does not check if the episode is watchable which is useful when importing episodes
    pub fn track_episode_unchecked(&mut self, episode_number: Episode) {
        self.episodes.insert(episode_number);
    }

    /// adds a range of episode to be tracked
    ///
    /// if all episodes in the range were newly added, true is returned. if atleast one episode was not newly
    /// added i.e. it existed already before adding, false is returned.
    pub async fn track_episodes(
        &mut self,
        series_id: u32,
        season_number: u32,
        episodes_range: RangeInclusive<u32>,
    ) -> AddResult {
        let mut already_added_items = 0;
        for episode_number in episodes_range.clone() {
            if !self
                .track_episode(series_id, season_number, episode_number)
                .await
            {
                already_added_items += 1;
            };
        }

        if already_added_items == 0 {
            AddResult::Full
        } else if already_added_items == episodes_range.count() {
            AddResult::None
        } else {
            AddResult::Partial
        }
    }

    pub fn untrack_episode(&mut self, episode: Episode) {
        self.episodes.remove(&episode);
    }

    pub fn is_episode_watched(&self, episode: Episode) -> bool {
        self.episodes.contains(&episode)
    }

    /// Return the last watched episode
    ///
    /// This obviously skip any unwatched episode in between and just returns the highest
    pub fn get_last_episode(&self) -> Option<Episode> {
        self.episodes.iter().max().copied()
    }

    /// Get the total amount of episodes in the season
    pub fn get_total_episodes(&self) -> usize {
        self.episodes.len()
    }
}

impl Default for Season {
    fn default() -> Self {
        Self::new()
    }
}

pub type Episode = u32;

/// Indicates if adding episodes has been fully added(when none of the episodes were present before adding) or
/// partial(when some were already present) and none when all the added apisode where already present
#[derive(Debug, Clone)]
pub enum AddResult {
    /// When adding is successfully done for all items
    Full,
    /// When adding is successfully done for some items
    Partial,
    /// When adding did not happen
    None,
}

pub mod database_transfer {
    //! Implementations of importing and exporting series tracking data

    use std::{io, path};

    use super::Series;
    use super::DB;

    use ron::ser;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    const CURRENT_DATA_VERSION: u16 = 1;

    #[derive(Debug, Error)]
    pub enum ImportError {
        #[error("IO error: {0}")]
        Io(io::Error),
        #[error("incompatible version. Expected version {0}, found {1}")]
        Version(u16, u16),
        #[error("deserialization error: {0}")]
        Deserialization(ron::de::SpannedError),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TransferData {
        version: u16,
        series: Vec<Series>,
    }

    impl TransferData {
        pub fn new(series: Vec<Series>) -> Self {
            Self {
                version: CURRENT_DATA_VERSION,
                series,
            }
        }

        fn error_when_incompatible(import_data_version: u16) -> Result<(), ImportError> {
            if import_data_version == CURRENT_DATA_VERSION {
                Ok(())
            } else {
                Err(ImportError::Version(
                    CURRENT_DATA_VERSION,
                    import_data_version,
                ))
            }
        }

        pub fn blocking_import(path: impl AsRef<path::Path>) -> Result<Self, ImportError> {
            let import = std::fs::read_to_string(path).map_err(ImportError::Io)?;
            let imported_data =
                ron::from_str::<Self>(&import).map_err(ImportError::Deserialization)?;

            Self::error_when_incompatible(imported_data.version).map(|_| imported_data)
        }

        pub fn blocking_import_to_db(path: impl AsRef<path::Path>) -> Result<(), ImportError> {
            DB.import(&Self::blocking_import(path)?);
            Ok(())
        }

        pub async fn async_import(path: impl AsRef<path::Path>) -> Result<Self, ImportError> {
            let import = tokio::fs::read_to_string(path)
                .await
                .map_err(ImportError::Io)?;
            let imported_data =
                ron::from_str::<Self>(&import).map_err(ImportError::Deserialization)?;

            Self::error_when_incompatible(imported_data.version).map(|_| imported_data)
        }

        pub async fn async_import_to_db(path: impl AsRef<path::Path>) -> Result<(), ImportError> {
            DB.import(&Self::async_import(path).await?);
            Ok(())
        }

        pub fn get_series(&self) -> &[Series] {
            &self.series
        }

        fn ron_str(&self) -> String {
            let pretty_config = ser::PrettyConfig::new().depth_limit(4);
            ser::to_string_pretty(self, pretty_config).expect("transfer data serialization")
        }

        pub fn blocking_export(&self, path: impl AsRef<path::Path>) -> Result<(), io::Error> {
            let ron_str = self.ron_str();
            std::fs::write(path, ron_str)
        }

        pub fn blocking_export_from_db(path: impl AsRef<path::Path>) -> Result<(), io::Error> {
            DB.export().blocking_export(path)
        }

        pub async fn async_export(&self, path: impl AsRef<path::Path>) -> Result<(), io::Error> {
            let ron_str = self.ron_str();
            tokio::fs::write(path, ron_str).await
        }

        pub async fn async_export_from_db(path: impl AsRef<path::Path>) -> Result<(), io::Error> {
            DB.export().async_export(path).await
        }
    }
}
