use crate::core::{api::tv_maze::series_information::SeriesMainInformation, caching};

use super::series_tree;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

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
    pub fn update(&self, db: sled::Db) {
        series_tree::add_series(db, self.id, self);
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

// impl Drop for Series {
//     fn drop(&mut self) {
//         // Making sure database series is updated
//         self.update();

//         // Preventing unwatched and untracked series from cloggin up the database.
//         // This can happen when a user adds a series for tracking and untracks the
//         // series without having any episodes checked.
//         if !self.is_tracked() && self.get_total_episodes() == 0 {
//             DB.remove_series(self.id);
//         }
//     }
// }

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
