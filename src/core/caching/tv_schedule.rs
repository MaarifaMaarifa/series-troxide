use std::collections::HashSet;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::api::tv_maze::tv_schedule::{get_episodes_with_country, get_episodes_with_date};
use crate::core::api::tv_maze::Rated;
use crate::core::posters_hiding::HIDDEN_SERIES;

pub mod full_schedule;

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

    let mut series_infos = deduplicate_items(series_infos)
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

    let mut series_infos = deduplicate_items(series_infos)
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
fn deduplicate_items<T>(series_infos: Vec<T>) -> Vec<T>
where
    T: std::cmp::Eq + std::hash::Hash,
{
    let set: HashSet<T> = series_infos.into_iter().collect();
    set.into_iter().collect()
}

/// Sorts the given slice of `SeriesMainInformation` starting from the one with highest rating to the lowest
fn sort_by_rating<T>(series_infos: &mut [T])
where
    T: Rated,
{
    series_infos.sort_unstable_by(|a, b| b.rating().total_cmp(&a.rating()));
}

async fn get_hidden_series_ids() -> HashSet<u32> {
    HIDDEN_SERIES
        .write()
        .await
        .get_hidden_series_ids()
        .await
        .unwrap_or_default()
}
