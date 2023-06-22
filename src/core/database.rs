use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use tracing::info;

const DATABASE_FOLDER_NAME: &str = "series-troxide-db";

lazy_static! {
    pub static ref DB: Database = Database::init();
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

    pub fn track_series(&self, series_id: u32, series: Series) {
        self.db
            .insert(series_id.to_string(), bincode::serialize(&series).unwrap())
            .unwrap();
    }

    pub fn untrack_series(&self, series_id: u32) {
        self.db.remove(series_id.to_string()).unwrap();
    }

    pub fn get_series(&self, series_id: u32) -> Option<Series> {
        let series_bytes = self.db.get(series_id.to_string()).unwrap()?;
        Some(bincode::deserialize(&series_bytes).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Series {
    name: String,
    seasons: HashMap<u32, Season>,
}

impl Series {
    pub fn new(name: String) -> Self {
        Self {
            name,
            seasons: HashMap::new(),
        }
    }

    pub fn add_season(&mut self, season_number: u32, season: Season) {
        self.seasons.insert(season_number, season);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    episodes: HashMap<u32, Episode>,
}

impl Season {
    pub fn new() -> Self {
        Self {
            episodes: HashMap::new(),
        }
    }

    pub fn track_episode(&mut self, episode_number: u32, episode: Episode) {
        self.episodes.insert(episode_number, episode);
    }

    pub fn untrack_episode(&mut self, episode_number: u32) {
        self.episodes.remove(&episode_number);
    }

    pub fn mark_watched(&mut self) {
        self.episodes
            .values_mut()
            .for_each(|episode| episode.mark_watched())
    }

    pub fn mark_unwatched(&mut self) {
        self.episodes
            .values_mut()
            .for_each(|episode| episode.mark_unwatched())
    }

    pub fn episodes_watched(&self) -> usize {
        self.episodes
            .values()
            .filter(|episode| episode.is_watched())
            .count()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    is_watched: bool,
    airstamp: String,
    average_watchtime: u32,
}

impl Episode {
    pub fn new(airstamp: String, average_watchtime: u32) -> Self {
        Self {
            airstamp,
            is_watched: false,
            average_watchtime,
        }
    }

    pub fn is_watched(&self) -> bool {
        self.is_watched
    }

    pub fn mark_watched(&mut self) {
        self.is_watched = true
    }

    pub fn mark_unwatched(&mut self) {
        self.is_watched = false
    }
}
