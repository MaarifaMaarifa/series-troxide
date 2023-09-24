use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};
use tracing::info;

use super::{api::tv_maze::series_information::SeriesMainInformation, caching};

/*
The last digit represents the version of the database.
This should correspond to the version of the file magic
of the exportation and importation file.
*/
const DATABASE_FOLDER_NAME: &str = "series-troxide-db-1";

lazy_static! {
    pub static ref DB: Database = Database::init();
}

/// This is a `Vec` containing keys corresponding to their values in the database
/// in their bytes form usefull for importing and exporting database data
pub type KeysValuesVec = Vec<(Vec<u8>, Vec<u8>)>;

pub fn get_ids_from_keys_values_vec(keys_values_vec: KeysValuesVec) -> Vec<String> {
    let mut series_ids = Vec::with_capacity(keys_values_vec.len());
    for (series_id, _) in keys_values_vec {
        let series_id = String::from_utf8_lossy(&series_id).to_string();
        series_ids.push(series_id)
    }
    series_ids
}

pub struct Database {
    db: Db,
}

impl Database {
    fn init() -> Self {
        info!("opening database");
        if let Some(proj_dir) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
            let mut database_path = std::path::PathBuf::from(&proj_dir.data_dir());
            database_path.push(DATABASE_FOLDER_NAME);
            let db = sled::open(database_path).unwrap();
            if !db.was_recovered() {
                info!("created a fresh database as none was found");
            }
            return Self { db };
        }
        panic!("could not get the path to database");
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

    /// get series ids and their corrensponding series structures
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

    /// Returns the bytes of the all database
    ///
    /// # Errors
    ///
    /// Export fails when reading from the database fails i.e io problems or when data fails to be
    /// serialize (super unlikely).
    pub fn export(&self) -> anyhow::Result<Vec<u8>> {
        let kv: Result<KeysValuesVec, sled::Error> = self
            .db
            .iter()
            .map(|kv| kv.map(|(key, value)| (key.to_vec(), value.to_vec())))
            .collect();
        Ok(bincode::serialize(&kv?)?)
    }

    /// Reads the bytes and adds them to the database
    ///
    /// # Note
    ///
    /// For already existing series, this will replace their data with the new one.
    ///
    /// # Errors
    ///
    /// Import fails when the bytes are invalid, when bytes insertion to the database fails
    /// i.e io problems, and when database fails to flush.
    pub fn import(&self, data: &[u8]) -> anyhow::Result<()> {
        let data: KeysValuesVec = bincode::deserialize(data)?;
        self.import_keys_value_vec(data)
    }

    pub fn import_keys_value_vec(&self, data: KeysValuesVec) -> anyhow::Result<()> {
        data.into_iter()
            .try_for_each(|(key, value)| self.db.insert(key, value).map(|_| ()))?;

        self.db.flush()?;
        Ok(())
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
    /// calling it unless if you want imediate update i.e. there is some code that
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
            if caching::episode_list::EpisodeList::is_episode_watchable(episode) == Some(true) {
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

    use super::DB;

    use std::ffi::OsStr;
    use std::fs;
    use std::path;

    /*
    The first 4 bytes "sro2" represent "seriestroxide"
    The last 4 bytes represent the version, "0001" will be version 1
    */
    const MAGIC: &[u8; 8] = b"sro20001";
    const CURRENT_DATA_VERSION: u16 = 1;

    const DEFAULT_DATABASE_EXPORT_NAME: &str = "series-troxide-export";

    /// # Reads series tracking data from the provided path
    ///
    /// This will directly import the data into the database
    pub fn read_database_from_path(database_read_path: &path::Path) -> anyhow::Result<()> {
        DB.import(&remove_magic_from_read_data(fs::read(database_read_path)?)?)?;
        Ok(())
    }

    /// # Reads series tracking data from the provided path and returns the `KeysValuesVec`
    ///
    /// This can be usefull when you want to cache the series data first before importing it to the database.
    pub fn read_database_from_path_as_keys_value_vec(
        database_read_path: &path::Path,
    ) -> anyhow::Result<super::KeysValuesVec> {
        let data = &remove_magic_from_read_data(fs::read(database_read_path)?)?;
        Ok(bincode::deserialize(data)?)
    }

    /// Writes series tracking data from the provided directory path
    ///
    /// Takes in an optional name to be used as a name for the exported data. Otherwise
    /// it defaults to series-troxide-export
    ///
    /// # Note
    /// This overwrites any file of the same name if it exists
    pub fn write_database_to_path(
        database_write_path: &path::Path,
        database_name: Option<&OsStr>,
    ) -> anyhow::Result<()> {
        let raw_data = DB.export()?;

        let mut database_write_path = path::PathBuf::from(database_write_path);

        if let Some(name) = database_name {
            database_write_path.push(name);
        } else {
            database_write_path.push(DEFAULT_DATABASE_EXPORT_NAME);
        }

        fs::write(database_write_path, add_magic_into_raw_data(raw_data))?;
        Ok(())
    }

    #[derive(Debug, thiserror::Error, PartialEq)]
    enum DataFormatError {
        #[error("invalid file")]
        InvalidMagic,

        #[error("invalid file version")]
        InvalidVersion,

        #[error("wrong version. expected version {0}, found version {0}")]
        WrongVersion(u16, u16),
    }

    /// Reads data from the path, checks magic if it's series troxide's and returns data
    /// that follow after the magic
    fn remove_magic_from_read_data(data: Vec<u8>) -> Result<Vec<u8>, DataFormatError> {
        if data[..MAGIC.len()] == MAGIC[..] {
            Ok(data[MAGIC.len()..].into())
        } else if data[..4] == MAGIC[..4] {
            match dbg!(String::from_utf8_lossy(&data[4..MAGIC.len()]).parse::<u16>()) {
                Ok(wrong_version) => Err(DataFormatError::WrongVersion(
                    CURRENT_DATA_VERSION,
                    wrong_version,
                )),
                Err(_) => Err(DataFormatError::InvalidVersion),
            }
        } else {
            Err(DataFormatError::InvalidMagic)
        }
    }

    /// Adds magic bytes into raw data
    fn add_magic_into_raw_data(mut raw_data: Vec<u8>) -> Vec<u8> {
        let mut magicfied_data: Vec<u8> = MAGIC.to_vec(); // Initialize data with magic
        magicfied_data.append(&mut raw_data);
        magicfied_data
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn remove_magic_from_read_data_test() {
            let mut raw_data = vec![1, 2, 3];
            let mut data: Vec<u8> = MAGIC.to_vec(); // Initialize data with magic
            data.append(&mut raw_data);

            // Checking if we get out actual data
            assert_eq!(vec![1, 2, 3], remove_magic_from_read_data(data).unwrap())
        }

        #[test]
        fn add_magic_into_raw_data_test() {
            let mut raw_data = vec![1, 2, 3];
            let magicfied_data = add_magic_into_raw_data(raw_data.clone());

            let mut data: Vec<u8> = MAGIC.to_vec(); // Initialize data with magic
            data.append(&mut raw_data);

            assert_eq!(magicfied_data, data);
        }

        #[test]
        fn valid_file_test() {
            assert!(remove_magic_from_read_data(b"sro20001".to_vec()).is_ok())
        }

        #[test]
        fn invalid_file_test() {
            assert_eq!(
                remove_magic_from_read_data(b"helloworld".to_vec()),
                Err(DataFormatError::InvalidMagic)
            )
        }

        #[test]
        fn invalid_version_test() {
            assert_eq!(
                remove_magic_from_read_data(b"sro2hello".to_vec()),
                Err(DataFormatError::InvalidVersion)
            )
        }

        #[test]
        fn wrong_version_test() {
            assert_eq!(
                remove_magic_from_read_data(b"sro29000".to_vec()),
                Err(DataFormatError::WrongVersion(CURRENT_DATA_VERSION, 9000))
            )
        }
    }
}
