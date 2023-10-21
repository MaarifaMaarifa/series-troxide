use std::{collections::HashSet, io::ErrorKind};

use crate::core::{
    api::tv_maze::{
        deserialize_json,
        episodes_information::{get_episode_list, Episode},
        ApiError,
    },
    caching::CACHER,
    database,
};
use chrono::{DateTime, Datelike, Duration, Local, Timelike, Utc};
use tracing::info;

use super::{read_cache, write_cache, CacheFilePath};

#[derive(Clone, Debug)]
pub struct EpisodeList {
    series_id: u32,
    episodes: Vec<Episode>,
}

impl EpisodeList {
    pub async fn new(series_id: u32) -> Result<Self, ApiError> {
        let episodes_list_path =
            CACHER.get_cache_file_path(CacheFilePath::SeriesEpisodeList(series_id));

        let json_string = match read_cache(&episodes_list_path).await {
            Ok(json_string) => json_string,
            Err(err) => {
                info!("falling back online for 'episode_list' for series id: {series_id}");
                let (episodes, json_string) = get_episode_list(series_id).await?;

                if err.kind() == ErrorKind::NotFound {
                    write_cache(&json_string, &episodes_list_path).await;
                }
                return Ok(Self {
                    series_id,
                    episodes,
                });
            }
        };

        let episodes = deserialize_json::<Vec<Episode>>(&json_string)?;
        Ok(Self {
            series_id,
            episodes,
        })
    }

    /// Constructs `EpisodeList` from it's cache file contents directly
    pub fn with_cache(series_id: u32, cache_str: &str) -> Result<Self, ApiError> {
        let episodes = deserialize_json::<Vec<Episode>>(cache_str)?;
        Ok(Self {
            series_id,
            episodes,
        })
    }

    pub fn get_episode(&self, season_number: u32, episode_number: u32) -> Option<&Episode> {
        self.episodes.iter().find(|episode| {
            (episode.season == season_number) && (episode.number == Some(episode_number))
        })
    }

    pub fn get_episodes(&self, season: u32) -> Vec<&Episode> {
        self.episodes
            .iter()
            .filter(|episode| episode.season == season)
            .collect()
    }

    pub fn get_all_episodes(&self) -> &[Episode] {
        &self.episodes
    }

    // /// Get the total number of all episodes in the Series
    // pub fn get_total_episodes(&self) -> usize {
    //     self.episodes.len()
    // }

    /// Get the total number of all watchable episodes in the Series
    pub fn get_total_watchable_episodes(&self) -> usize {
        self.episodes
            .iter()
            .filter(|episode| Self::is_episode_watchable(episode) == Some(true))
            .count()
    }

    /// Returns the number of all seasons available and their total episodes as a tuple (season_no, total_episodes)
    pub fn get_season_numbers_with_total_episode(&self) -> Vec<(u32, TotalEpisodes)> {
        let seasons: HashSet<u32> = self.episodes.iter().map(|episode| episode.season).collect();
        let mut seasons: Vec<u32> = seasons.iter().copied().collect();
        seasons.sort();

        seasons
            .into_iter()
            .map(|season| {
                let total_episodes = self.get_episodes(season).len();
                let total_watchable_episodes = self
                    .get_episodes(season)
                    .into_iter()
                    .filter(|episode| Self::is_episode_watchable(episode) == Some(true))
                    .count();
                (
                    season,
                    TotalEpisodes::new(total_episodes, total_watchable_episodes),
                )
            })
            .collect()
    }

    /// Returns the number of all seasons available and their total episodes as a tuple (season_no, total_episodes)
    pub fn get_season_numbers_with_total_watchable_episode(&self) -> Vec<(u32, usize)> {
        let seasons: HashSet<u32> = self.episodes.iter().map(|episode| episode.season).collect();
        let mut seasons: Vec<u32> = seasons.iter().copied().collect();
        seasons.sort();

        seasons
            .into_iter()
            .map(|season| {
                let total_episodes = self
                    .get_episodes(season)
                    .into_iter()
                    .filter(|episode| Self::is_episode_watchable(episode) == Some(true))
                    .count();
                (season, total_episodes)
            })
            .collect()
    }

    /// Tells if the episode is watchable or not based on the current time and the episode release time
    ///
    /// This method returns an optional bool as an episode my not have airstamp associated with it hence
    /// the method can not infer that information.
    pub fn is_episode_watchable(episode: &Episode) -> Option<bool> {
        let airstamp = DateTime::parse_from_rfc3339(episode.airstamp.as_ref()?)
            .unwrap()
            .with_timezone(&Local);
        let local_time = Utc::now().with_timezone(&Local);
        Some(airstamp <= local_time)
    }

    /// Returns the previous episode to air from the current time
    ///
    /// This method is also useful when finding the maximum watchable episode
    /// as you can not watch an episode that is released in the future.
    pub fn get_previous_episode_to_air(&self) -> Option<&Episode> {
        let mut episodes_iter = self.episodes.iter().peekable();
        while let Some(episode) = episodes_iter.next() {
            if let Some(peeked_episode) = episodes_iter.peek() {
                if !Self::is_episode_watchable(peeked_episode)? {
                    return Some(episode);
                }
            } else {
                return Some(episode);
            }
        }
        None
    }

    /// Returns the next episode to air from the current time
    pub fn get_next_episode_to_air(&self) -> Option<&Episode> {
        self.episodes
            .iter()
            .find(|episode| Self::is_episode_watchable(episode) == Some(false))
    }

    /// Returns the next episode to air and it's release time
    pub fn get_next_episode_to_air_and_time(&self) -> Option<(&Episode, EpisodeReleaseTime)> {
        let next_episode = self.get_next_episode_to_air()?;
        let local_date_time = next_episode.local_date_time().ok()?;

        let release_time = EpisodeReleaseTime::new(local_date_time);
        Some((next_episode, release_time))
    }

    pub fn get_next_episode_to_watch(&self) -> Option<&Episode> {
        let series = database::DB
            .get_series(self.series_id)
            .expect("series not in the database");

        self.get_all_episodes()
            .iter()
            .filter(|episode| Self::is_episode_watchable(episode) == Some(true))
            .find(|episode| {
                series
                    .get_season(episode.season)
                    .map(|season| {
                        episode
                            .number
                            .map(|episode_number| !season.is_episode_watched(episode_number))
                            .unwrap_or(false)
                    })
                    .unwrap_or(true) // if season isn't watched, let's get it's first episode
            })
    }
}

#[derive(Clone, Debug)]
pub struct TotalEpisodes {
    all_episodes: usize,
    all_watchable_episodes: usize,
}

impl TotalEpisodes {
    fn new(all_episodes: usize, all_watchable_episodes: usize) -> Self {
        Self {
            all_episodes,
            all_watchable_episodes,
        }
    }

    /// Retrieves all the episodes
    pub fn get_all_episodes(&self) -> usize {
        self.all_episodes
    }

    /// Retrieves all the watchable episodes
    pub fn get_all_watchable_episodes(&self) -> usize {
        self.all_watchable_episodes
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct EpisodeReleaseTime {
    release_time: DateTime<Local>,
}

impl EpisodeReleaseTime {
    pub fn new(release_time: DateTime<Local>) -> Self {
        Self { release_time }
    }

    pub fn get_remaining_release_duration(&self) -> Duration {
        let local_time = Utc::now().with_timezone(&Local);
        self.release_time - local_time
    }

    /// Returns the remaining full date and time for an episode to be released
    pub fn get_full_release_date_and_time(&self) -> String {
        /// appends zero the minute digit if it's below 10 for better display
        fn append_zero(num: u32) -> String {
            if num < 10 {
                format!("0{num}")
            } else {
                format!("{num}")
            }
        }

        let (is_pm, hour) = self.release_time.hour12();
        let pm_am = if is_pm { "p.m." } else { "a.m." };

        let minute = append_zero(self.release_time.minute());

        format!(
            "{} {} {}:{} {}",
            self.release_time.date_naive(),
            self.release_time.weekday(),
            hour,
            minute,
            pm_am
        )
    }
}
