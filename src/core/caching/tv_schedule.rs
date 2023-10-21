use std::collections::HashSet;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::api::tv_maze::tv_schedule::{get_episodes_with_country, get_episodes_with_date};
use crate::core::posters_hiding::HIDDEN_SERIES;

/// Retrieves series aired on a specific date through the provided optional &str
/// If None is supplied, it will default the the current day
///
/// ## Note
/// Expect slightly different results for the when calling multiple times with very small time gap,
/// this is because this function uses a `HashSet` for deduplication since duplicates
/// can appear at any random indices(not necessarily consecutive).
/// Sorts the collection from the one with highest rating to the lowest.
pub async fn get_series_with_date(
    date: Option<&str>,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_date(date).await?;
    let series_infos = get_series_infos_from_episodes(episodes).await?;

    let hidden_series_ids = get_hidden_series_ids().await;

    let mut series_infos = deduplicate_series_infos(series_infos)
        .into_iter()
        .filter(|series| hidden_series_ids.get(&series.id).is_none())
        .collect::<Vec<SeriesMainInformation>>();

    sort_by_rating(&mut series_infos);
    Ok(series_infos)
}

/// # Retrieves series aired on the current day at a particular country provided in ISO 3166-1
///
/// ## Note
/// Expect slightly different results for the when calling multiple times with very small time gap,
/// this is because this function uses a `HashSet` for deduplication since duplicates
/// can appear at any random indices(not necessarily consecutive).
/// Sorts the collection from the one with highest rating to the lowest.
///
/// Excludes hidden series
pub async fn get_series_with_country(
    country_iso: &str,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_country(country_iso).await?;

    let series_infos = get_series_infos_from_episodes(episodes).await?;

    let hidden_series_ids = get_hidden_series_ids().await;

    let mut series_infos = deduplicate_series_infos(series_infos)
        .into_iter()
        .filter(|series| hidden_series_ids.get(&series.id).is_none())
        .collect::<Vec<SeriesMainInformation>>();

    sort_by_rating(&mut series_infos);
    Ok(series_infos)
}

/// # Get `SeriesInformation`s from `Episode`s
///
/// Before acquiring the `SeriesInformation`s online, this function will attempt to check if each episode has
/// any embedded `SeriesInformation` and use that instead of requesting it online.
async fn get_series_infos_from_episodes(
    episodes: Vec<Episode>,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let mut episodes: Vec<Option<Episode>> = episodes.into_iter().map(Some).collect();

    let mut all_series_infos: Vec<SeriesMainInformation> = Vec::new();

    // Dealing with episodes with `Some` variants in their `show` field
    let mut series_infos: Vec<_> = episodes
        .iter_mut()
        .filter(|episode| {
            episode
                .as_ref()
                .map(|episode| episode.show.is_some())
                .unwrap_or_default()
        })
        .filter_map(|episode| episode.take())
        .filter_map(|episode| episode.show)
        .collect();
    all_series_infos.append(&mut series_infos);

    // Dealing with episodes with `Some` variants in their `embedded` field
    let mut series_infos: Vec<_> = episodes
        .iter_mut()
        .filter(|episode| {
            episode
                .as_ref()
                .map(|episode| episode.embedded.is_some())
                .unwrap_or_default()
        })
        .filter_map(|episode| episode.take())
        .filter_map(|episode| episode.embedded)
        .map(|embedded| embedded.show)
        .collect();
    all_series_infos.append(&mut series_infos);

    // Requesting online for any left over episodes
    let handles: Vec<_> = episodes
        .into_iter()
        .filter_map(|mut episode| episode.take())
        .map(|episode| {
            tokio::spawn(super::series_information::get_series_main_info_with_url(
                episode.links.show.href,
            ))
        })
        .collect();

    let mut series_infos = Vec::with_capacity(handles.len());
    for handle in handles {
        series_infos.push(handle.await??)
    }
    all_series_infos.append(&mut series_infos);

    Ok(all_series_infos)
}

/// # Remove duplicates from a `SeriesMainInformation` collection
///
/// Expect slightly different results for the same provided collection, this is
/// because this function uses a `HashSet` for deduplication since duplicates
/// can appear at any random indices(not necessarily consecutive)
fn deduplicate_series_infos(
    series_infos: Vec<SeriesMainInformation>,
) -> Vec<SeriesMainInformation> {
    let unique_set: HashSet<SeriesMainInformation> = series_infos.into_iter().collect();
    unique_set.into_iter().collect()
}

/// Sorts the given slice of `SeriesMainInformation` starting from the one with highest rating to the lowest
fn sort_by_rating(series_infos: &mut [SeriesMainInformation]) {
    series_infos.sort_unstable_by(|series_a, series_b| {
        series_b
            .rating
            .average
            .map(|rating| rating as u32)
            .unwrap_or(0)
            .cmp(
                &series_a
                    .rating
                    .average
                    .map(|rating| rating as u32)
                    .unwrap_or(0),
            )
    });
}

async fn get_hidden_series_ids() -> HashSet<u32> {
    HIDDEN_SERIES
        .write()
        .await
        .get_hidden_series_ids()
        .await
        .unwrap_or_default()
}

pub mod full_schedule {
    use std::collections::HashSet;

    use anyhow::{bail, Context};
    use chrono::{Datelike, Local, NaiveDate};
    use tokio::fs;
    use tokio::sync::{OnceCell, RwLock};
    use tracing::{error, info};

    use crate::core::api::tv_maze::deserialize_json;
    use crate::core::api::tv_maze::episodes_information::Episode;
    use crate::core::api::tv_maze::series_information::{
        Genre, SeriesMainInformation, ShowNetwork, ShowWebChannel,
    };
    use crate::core::api::tv_maze::tv_schedule::get_full_schedule;
    use crate::core::caching::CACHER;

    const FULL_SCHEDULE_CACHE_FILENAME: &str = "full-schedule";

    static FULL_SCHEDULE: OnceCell<FullSchedule> = OnceCell::const_new();
    static HIDDEN_SERIES_IDS: RwLock<Option<HashSet<u32>>> = RwLock::const_new(None);

    fn is_hidden(id: u32) -> bool {
        HIDDEN_SERIES_IDS
            .blocking_read()
            .as_ref()
            .map(|hidden_series_id| hidden_series_id.get(&id).is_some())
            .unwrap_or_default()
    }

    /// `FullSchedule` is a list of all future episodes known to TVmaze, regardless of their country.
    #[derive(Clone, Debug)]
    pub struct FullSchedule {
        episodes: Vec<Episode>,
    }

    impl FullSchedule {
        pub async fn new<'a>() -> anyhow::Result<&'a Self> {
            let hidden_series_ids = super::get_hidden_series_ids().await;

            if FULL_SCHEDULE.initialized() {
                if Some(&hidden_series_ids) != HIDDEN_SERIES_IDS.read().await.as_ref() {
                    *HIDDEN_SERIES_IDS.write().await = Some(hidden_series_ids);
                }
            } else {
                *HIDDEN_SERIES_IDS.write().await = Some(hidden_series_ids);
            }

            FULL_SCHEDULE
                .get_or_try_init(|| async { Self::load().await })
                .await
        }

        async fn load() -> anyhow::Result<Self> {
            let mut cache_path = CACHER.get_root_cache_path().to_owned();
            cache_path.push(FULL_SCHEDULE_CACHE_FILENAME);

            match cache_path.metadata() {
                Ok(metadata) => match metadata.created() {
                    Ok(sys_time) => {
                        let daily_schedule_age = sys_time.elapsed().unwrap_or_else(|err| {
                            error!("failed to get daily episode schedule age: {}", err);
                            std::time::Duration::default()
                        });
                        if daily_schedule_age > std::time::Duration::from_secs(24 * 60 * 60) {
                            info!("cleaning outdated daily episode schedule");
                            fs::remove_file(&cache_path).await.unwrap_or_else(|err| {
                                error!("failed to clean outdated daily episode schedule: {}", err)
                            });
                        }
                    }
                    Err(err) => error!(
                        "failed to get daily episode schedule time of creating: {}",
                        err
                    ),
                },
                Err(err) => error!("failed to get daily episode schedule metadata: {}", err),
            }

            let json_string = match fs::read_to_string(&cache_path).await {
                Ok(json_string) => json_string,
                Err(err) => {
                    if let std::io::ErrorKind::NotFound = err.kind() {
                        info!("downloading daily episode schedule");
                        let cache_str = get_full_schedule()
                            .await
                            .context("failed to download daily episode schedule")?;
                        fs::write(cache_path, &cache_str)
                            .await
                            .context("failed to save daily episode schedule")?;
                        cache_str
                    } else {
                        bail!(
                            "critical error when reading daily episode schedule: {}",
                            err
                        )
                    }
                }
            };

            let episodes = deserialize_json::<Vec<Episode>>(&json_string)?;
            Ok(Self { episodes })
        }

        /// # Returns new series aired in the given month
        ///
        /// These are series premieres airing for the first time.
        /// takes in an amount describing how many of `SeriesMainInformation` to return since they can
        /// be alot
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_monthly_new_series(
            &self,
            amount: usize,
            month: chrono::Month,
        ) -> Vec<SeriesMainInformation> {
            self.get_monthly_series_with_condition(amount, month, |episode| {
                episode.number.map(|num| num == 1).unwrap_or_default() && episode.season == 1
            })
        }

        /// # Returns returning series aired in the given month
        ///
        /// These are series premieres starting from the second season.
        /// takes in an amount describing how many of `SeriesMainInformation` to return since they can
        /// be alot
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at at random indices(not necessarily consecutive)
        pub fn get_monthly_returning_series(
            &self,
            amount: usize,
            month: chrono::Month,
        ) -> Vec<SeriesMainInformation> {
            self.get_monthly_series_with_condition(amount, month, |episode| {
                episode.number.map(|num| num == 1).unwrap_or_default() && episode.season != 1
            })
        }

        /// # Returns popular series filtered out using the provided genre
        ///
        /// ## Note
        /// - Less accurate as it priotizes rating of a show. `self().get_series_by_genres` is more accurate
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_genre(
            &self,
            amount: Option<usize>,
            genre: &Genre,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_with_condition(amount, |series_info| {
                series_info
                    .get_genres()
                    .into_iter()
                    .any(|series_genre| series_genre == *genre)
            })
        }

        /// # Returns popular series filtered out using the provided genre
        ///
        /// ## Note
        /// - more accurate version of `self.get_popular_series_by_genre()` without caring of the rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_series_by_genres(
            &self,
            amount: usize,
            genres: &[Genre],
        ) -> Vec<SeriesMainInformation> {
            let mut counted_series = Self::get_genre_weight_for_series_information(
                self.get_popular_series(None),
                genres,
            );
            counted_series.sort_unstable_by(|(a, _), (b, _)| b.cmp(a));

            counted_series
                .into_iter()
                .take(amount)
                .filter(|(count, _)| *count > 0)
                .map(|(_, series_info)| series_info)
                .collect()
        }

        /// Return `SeriesMainInformation` and it's associated count of how many the supplied `genres` appeared
        /// in it's own genres.
        fn get_genre_weight_for_series_information(
            series_infos: Vec<SeriesMainInformation>,
            genres: &[Genre],
        ) -> Vec<(i32, SeriesMainInformation)> {
            fn calc_genre_weight(genres_a: &[Genre], genres_b: &[Genre]) -> i32 {
                let mut weight = 0;
                for b in genres_b {
                    if genres_a.iter().any(|a| a == b) {
                        weight += 1;
                    } else {
                        weight -= 1;
                    }
                }
                weight
            }

            series_infos
                .into_iter()
                .map(|series_info| {
                    (
                        calc_genre_weight(&series_info.get_genres(), genres),
                        series_info,
                    )
                })
                .collect()
        }

        /// # Returns popular series filtered out using the provided list of genres
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_genres(
            &self,
            amount: Option<usize>,
            genres: &[Genre],
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_with_condition(amount, |series_info| {
                series_info
                    .get_genres()
                    .into_iter()
                    .any(|series_genre| genres.iter().any(|genre| *genre == series_genre))
            })
        }

        /// # Returns popular series filtered out using the provided network
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_network(
            &self,
            amount: Option<usize>,
            network: &ShowNetwork,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_with_condition(amount, |series_info| {
                series_info
                    .get_network()
                    .map(|show_network| show_network == *network)
                    .unwrap_or_default()
            })
        }

        /// # Returns popular series filtered out using the provided webchannel
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_webchannel(
            &self,
            amount: Option<usize>,
            webchannel: &ShowWebChannel,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_with_condition(amount, |series_info| {
                series_info
                    .get_webchannel()
                    .map(|show_webchannel| show_webchannel == *webchannel)
                    .unwrap_or_default()
            })
        }

        /// # This is a list of all future series known to TVmaze, regardless of their country sorted by rating starting from the highest to the lowest
        ///
        /// takes in an amount describing how many of `SeriesMainInformation` to return since they can
        /// be alot
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_popular_series(&self, amount: Option<usize>) -> Vec<SeriesMainInformation> {
            self.get_popular_series_with_condition(amount, |_| true)
        }

        /// # This is a list of all future series known to TVmaze, regardless of their country sorted by rating starting from the highest to the lowest
        ///
        /// takes in an amount describing how many of `SeriesMainInformation` to return since they can
        /// be alot and a condition to filter out `SeriesMainInformation`
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        fn get_popular_series_with_condition<'a, F>(
            &self,
            amount: Option<usize>,
            condition: F,
        ) -> Vec<SeriesMainInformation>
        where
            F: 'a + Fn(&SeriesMainInformation) -> bool,
        {
            let mut series_infos = self.get_series_with_condition(condition);
            super::sort_by_rating(&mut series_infos);
            if let Some(amount) = amount {
                series_infos.into_iter().take(amount).collect()
            } else {
                series_infos
            }
        }

        /// # This is a list of all future series known to TVmaze, regardless of their country
        ///
        /// Takes a condition to filter out `SeriesMainInformation`
        ///
        /// ## Note
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        fn get_series_with_condition<'a, F>(&self, condition: F) -> Vec<SeriesMainInformation>
        where
            F: 'a + Fn(&SeriesMainInformation) -> bool,
        {
            self.episodes
                .iter()
                .filter_map(|episode| episode.embedded.as_ref())
                .cloned()
                .map(|embedded| embedded.show)
                .filter(condition)
                .filter(|series| !is_hidden(series.id))
                .collect::<HashSet<SeriesMainInformation>>()
                .into_iter()
                .collect()
        }

        /// # This is a list of all future series known to TVmaze, regardless of their country
        ///
        /// ## Note
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        pub fn get_series(&self) -> Vec<SeriesMainInformation> {
            self.episodes
                .iter()
                .filter_map(|episode| episode.embedded.as_ref())
                .cloned()
                .map(|embedded| embedded.show)
                .filter(|series| !is_hidden(series.id))
                .collect::<HashSet<SeriesMainInformation>>()
                .into_iter()
                .collect()
        }

        /// # Returns series aired in the given month with a given condition to be applied to episodes
        ///
        /// This condition filters out the aired episodes based on how it is described as the series
        /// are constructed from the aired episodes.
        /// Also takes in an amount describing how many of `SeriesMainInformation` to return since they can
        /// be alot
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear at any random indices(not necessarily consecutive)
        fn get_monthly_series_with_condition<'a, F>(
            &self,
            amount: usize,
            month: chrono::Month,
            condition: F,
        ) -> Vec<SeriesMainInformation>
        where
            F: 'a + Fn(&Episode) -> bool,
        {
            let current_year = Local::now().year();
            let month = month.number_from_month();
            let first_date_of_current_month =
                NaiveDate::from_ymd_opt(current_year, month, 1).expect("the date should be valid!");

            let all_dates_of_month: Vec<NaiveDate> =
                first_date_of_current_month.iter_days().take(30).collect();

            let episodes: Vec<Episode> = self
                .episodes
                .iter()
                .filter(|episode| condition(episode))
                .filter(|episode| {
                    episode
                        .date_naive()
                        .map(|naive_date| all_dates_of_month.iter().any(|date| *date == naive_date))
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            let mut series_infos: Vec<SeriesMainInformation> = super::deduplicate_series_infos(
                episodes
                    .into_iter()
                    .filter_map(|episode| episode.embedded)
                    .map(|embedded| embedded.show)
                    .filter(|series| !is_hidden(series.id))
                    .collect(),
            );

            super::sort_by_rating(&mut series_infos);

            series_infos.into_iter().take(amount).collect()
        }

        pub fn get_daily_global_series(&self, amount: usize) -> Vec<SeriesMainInformation> {
            self.get_series_by_date_with_condition(amount, Local::now().date_naive(), |_| true)
        }

        pub fn get_daily_local_series(
            &self,
            amount: usize,
            country_iso: &str,
        ) -> Vec<SeriesMainInformation> {
            self.get_series_by_date_with_condition(
                amount,
                Local::now().date_naive(),
                |series_info| series_info.get_country_code() == Some(country_iso),
            )
        }

        fn get_series_by_date_with_condition<'a, F>(
            &self,
            amount: usize,
            date: chrono::NaiveDate,
            condition: F,
        ) -> Vec<SeriesMainInformation>
        where
            F: 'a + Fn(&SeriesMainInformation) -> bool,
        {
            let episodes: Vec<Episode> = self
                .episodes
                .iter()
                .filter(|episode| {
                    episode
                        .date_naive()
                        .map(|naive_date| date == naive_date)
                        .unwrap_or_default()
                })
                .cloned()
                .collect();

            let mut series_infos: Vec<SeriesMainInformation> = super::deduplicate_series_infos(
                episodes
                    .into_iter()
                    .filter_map(|episode| episode.embedded)
                    .map(|embedded| embedded.show)
                    .filter(condition)
                    .filter(|series| !is_hidden(series.id))
                    .collect(),
            );

            super::sort_by_rating(&mut series_infos);

            series_infos.into_iter().take(amount).collect()
        }
    }
}
