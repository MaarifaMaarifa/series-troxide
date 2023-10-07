use std::collections::HashSet;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::api::tv_maze::tv_schedule::{get_episodes_with_country, get_episodes_with_date};

/// Retrieves series aired on a specific date through the provided optional &str
/// If None is supplied, it will default the the current day
///
/// ## Note
/// Expect slightly different results for the when calling multiple times with very small time gap,
/// this is because this function uses a `HashSet` for deduplication since duplicates
/// can appear and any random indices(not necessarily consecutive).
/// Sorts the collection from the one with highest rating to the lowest.
pub async fn get_series_with_date(
    date: Option<&str>,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_date(date).await?;
    let series_infos = get_series_infos_from_episodes(episodes).await?;
    let mut series_infos = deduplicate_series_infos(series_infos);
    sort_by_rating(&mut series_infos);
    Ok(series_infos)
}

/// # Retrieves series aired on the current day at a particular country provided in ISO 3166-1
///
/// ## Note
/// Expect slightly different results for the when calling multiple times with very small time gap,
/// this is because this function uses a `HashSet` for deduplication since duplicates
/// can appear and any random indices(not necessarily consecutive).
/// Sorts the collection from the one with highest rating to the lowest.
pub async fn get_series_with_country(
    country_iso: &str,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_country(country_iso).await?;
    let series_infos = get_series_infos_from_episodes(episodes).await?;
    let mut series_infos = deduplicate_series_infos(series_infos);
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
                .unwrap_or(false)
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
                .unwrap_or(false)
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
/// can appear and any random indices(not necessarily consecutive)
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

pub mod full_schedule {
    use std::collections::HashSet;

    use anyhow::{bail, Context};
    use chrono::{Datelike, Local, NaiveDate};
    use tokio::fs;
    use tracing::{error, info, warn};

    use crate::core::api::tv_maze::deserialize_json;
    use crate::core::api::tv_maze::episodes_information::Episode;
    use crate::core::api::tv_maze::series_information::{
        Genre, SeriesMainInformation, ShowNetwork, ShowWebChannel,
    };
    use crate::core::api::tv_maze::tv_schedule::get_full_schedule;
    use crate::core::caching::CACHER;
    use crate::core::posters_hiding::HIDDEN_SERIES;

    const FULL_SCHEDULE_CACHE_FILENAME: &str = "full-schedule";

    struct Filter<'a>(ScheduleFilter<'a>);

    enum ScheduleFilter<'a> {
        Network(&'a ShowNetwork),
        WebChannel(&'a ShowWebChannel),
        Genre(&'a Genre),
        Genres(Vec<Genre>),
        None,
    }

    /// `FullSchedule` is a list of all future episodes known to TVmaze, regardless of their country.
    #[derive(Clone, Debug)]
    pub struct FullSchedule {
        episodes: Vec<Episode>,
        hidden_series_ids: HashSet<u32>,
    }

    impl FullSchedule {
        /// Constructs `FullSchedule`
        pub async fn new() -> anyhow::Result<Self> {
            let mut cache_path = CACHER.get_root_cache_path().to_owned();
            cache_path.push(FULL_SCHEDULE_CACHE_FILENAME);

            let hidden_series_ids = Self::get_hidden_series_ids().await.unwrap_or_default();

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
            Ok(Self {
                episodes,
                hidden_series_ids,
            })
        }

        async fn get_hidden_series_ids() -> Option<HashSet<u32>> {
            let mut hidden_series = HIDDEN_SERIES.write().await;

            hidden_series
                .load_series()
                .await
                .map_err(|err| warn!("could not load hidden posters: {}", err))
                .ok()?;

            hidden_series
                .get_hidden_series()
                .map(|hidden_series| hidden_series.keys().copied().collect::<HashSet<u32>>())
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
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_monthly_new_series(
            &self,
            amount: usize,
            month: chrono::Month,
        ) -> Vec<SeriesMainInformation> {
            self.get_monthly_series(amount, month, |episode| {
                episode.number.map(|num| num == 1).unwrap_or(false) && episode.season == 1
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
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_monthly_returning_series(
            &self,
            amount: usize,
            month: chrono::Month,
        ) -> Vec<SeriesMainInformation> {
            self.get_monthly_series(amount, month, |episode| {
                episode.number.map(|num| num == 1).unwrap_or(false) && episode.season != 1
            })
        }

        /// # Returns popular series filtered out using the provided genre
        ///
        /// ## Note
        /// - Less accurate as it priotizes rating of a show. `self().get_series_by_genres` is more accurate
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_genre(
            &self,
            amount: usize,
            genre: &Genre,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_by_schedule_filter(amount, Filter(ScheduleFilter::Genre(genre)))
        }

        /// # Returns popular series filtered out using the provided genre
        ///
        /// ## Note
        /// - more accurate version of `self.get_popular_series_by_genre()` without caring of the rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_series_by_genres(
            &self,
            amount: usize,
            genres: &[Genre],
        ) -> Vec<SeriesMainInformation> {
            let mut counted_series =
                Self::get_genre_count_for_series_information(self.get_series(), genres);
            counted_series.sort_unstable_by(|(a, _), (b, _)| b.cmp(a));

            counted_series
                .into_iter()
                .take(amount)
                .map(|(_, series_info)| series_info)
                .collect()
        }

        /// Return `SeriesMainInformation` and it's associated count of how many the supplied `genres` appeared
        /// in it's own genres.
        fn get_genre_count_for_series_information(
            series_infos: Vec<SeriesMainInformation>,
            genres: &[Genre],
        ) -> Vec<(usize, SeriesMainInformation)> {
            series_infos
                .into_iter()
                .map(|series_info| {
                    (
                        series_info
                            .get_genres()
                            .into_iter()
                            .filter_map(|series_genre| {
                                genres.iter().find(|genre| **genre == series_genre)
                            })
                            .count(),
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
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_genres(
            &self,
            amount: usize,
            genres: Vec<Genre>,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_by_schedule_filter(
                amount,
                Filter(ScheduleFilter::Genres(genres)),
            )
        }

        /// # Returns popular series filtered out using the provided network
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_network(
            &self,
            amount: usize,
            network: &ShowNetwork,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_by_schedule_filter(
                amount,
                Filter(ScheduleFilter::Network(network)),
            )
        }

        /// # Returns popular series filtered out using the provided webchannel
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_popular_series_by_webchannel(
            &self,
            amount: usize,
            webchannel: &ShowWebChannel,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_by_schedule_filter(
                amount,
                Filter(ScheduleFilter::WebChannel(webchannel)),
            )
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
        ///   can appear and any random indices(not necessarily consecutive)
        pub fn get_popular_series(&self, amount: usize) -> Vec<SeriesMainInformation> {
            self.get_popular_series_by_schedule_filter(amount, Filter(ScheduleFilter::None))
        }

        /// # This is a list of all future series known to TVmaze, regardless of their country sorted by rating starting from the highest to the lowest
        ///
        /// takes in an amount describing how many of `SeriesMainInformation` to return since they can
        /// be alot. Also takes a Filter for filtering what `SeriesMainInformation` are requied
        ///
        /// ## Note
        /// - the returned collection is automatically sorted starting from series with highest rating.
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear and any random indices(not necessarily consecutive)
        fn get_popular_series_by_schedule_filter(
            &self,
            amount: usize,
            filter: Filter,
        ) -> Vec<SeriesMainInformation> {
            self.get_popular_series_with_condition(
                amount,
                filter,
                |series_info, schedule_filter| match schedule_filter {
                    ScheduleFilter::Network(network) => series_info
                        .get_network()
                        .map(|show_network| show_network == **network)
                        .unwrap_or(false),
                    ScheduleFilter::WebChannel(webchannel) => series_info
                        .get_webchannel()
                        .map(|show_webchannel| show_webchannel == **webchannel)
                        .unwrap_or(false),
                    ScheduleFilter::Genre(genre) => series_info
                        .get_genres()
                        .into_iter()
                        .any(|series_genre| series_genre == **genre),
                    ScheduleFilter::Genres(genres) => series_info
                        .get_genres()
                        .into_iter()
                        .any(|series_genre| genres.iter().any(|genre| *genre == series_genre)),
                    ScheduleFilter::None => true,
                },
            )
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
        ///   can appear and any random indices(not necessarily consecutive)
        fn get_popular_series_with_condition(
            &self,
            amount: usize,
            filter: Filter,
            filter_condition: fn(&SeriesMainInformation, &ScheduleFilter) -> bool,
        ) -> Vec<SeriesMainInformation> {
            let mut series_infos = self.get_series_with_condition(filter, filter_condition);
            super::sort_by_rating(&mut series_infos);
            series_infos.into_iter().take(amount).collect()
        }

        /// # This is a list of all future series known to TVmaze, regardless of their country
        ///
        /// Takes a condition to filter out `SeriesMainInformation`
        ///
        /// ## Note
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear and any random indices(not necessarily consecutive)
        fn get_series_with_condition(
            &self,
            filter: Filter,
            filter_condition: fn(&SeriesMainInformation, &ScheduleFilter) -> bool,
        ) -> Vec<SeriesMainInformation> {
            self.episodes
                .iter()
                .filter_map(|episode| episode.embedded.as_ref())
                .cloned()
                .map(|embedded| embedded.show)
                .filter(|series_info| filter_condition(series_info, &filter.0))
                .filter(|series| self.hidden_series_ids.get(&series.id).is_none())
                .collect::<HashSet<SeriesMainInformation>>()
                .into_iter()
                .collect()
        }

        /// # This is a list of all future series known to TVmaze, regardless of their country
        ///
        /// ## Note
        /// - Expect slightly different results for the same provided collection, this is
        ///   because this function uses a `HashSet` for deduplication since duplicates
        ///   can appear and any random indices(not necessarily consecutive)
        fn get_series(&self) -> Vec<SeriesMainInformation> {
            self.episodes
                .iter()
                .filter_map(|episode| episode.embedded.as_ref())
                .cloned()
                .map(|embedded| embedded.show)
                .filter(|series| self.hidden_series_ids.get(&series.id).is_none())
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
        ///   can appear and any random indices(not necessarily consecutive)
        fn get_monthly_series(
            &self,
            amount: usize,
            month: chrono::Month,
            filter_condition: fn(&Episode) -> bool,
        ) -> Vec<SeriesMainInformation> {
            let current_year = Local::now().year();
            let month = month.number_from_month();
            let first_date_of_current_month =
                NaiveDate::from_ymd_opt(current_year, month, 1).expect("the date should be valid!");

            let all_dates_of_month: Vec<NaiveDate> =
                first_date_of_current_month.iter_days().take(30).collect();

            let episodes: Vec<Episode> = self
                .episodes
                .iter()
                .filter(|episode| filter_condition(episode))
                .filter(|episode| {
                    all_dates_of_month
                        .iter()
                        .any(|date| *date == episode.get_naive_date().unwrap())
                })
                .cloned()
                .collect();

            let mut series_infos: Vec<SeriesMainInformation> = super::deduplicate_series_infos(
                episodes
                    .into_iter()
                    .filter_map(|episode| episode.embedded)
                    .map(|embedded| embedded.show)
                    .filter(|series| self.hidden_series_ids.get(&series.id).is_none())
                    .collect(),
            );

            super::sort_by_rating(&mut series_infos);

            series_infos.into_iter().take(amount).collect()
        }
    }
}
