use crate::core::api::episodes_information::Episode;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::tv_schedule::{get_episodes_with_country, get_episodes_with_date};

/// Retrieves series aired on a specific date through the provided optional &str
/// If None is supplied, it will default the the current day
pub async fn get_series_with_date(
    date: Option<&str>,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_date(date).await?;
    get_series_infos_from_episodes(episodes).await
}

/// Retrieves series aired on the current day at a particular country provided in ISO 3166-1
pub async fn get_series_with_country(
    country_iso: &str,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_country(country_iso).await?;
    get_series_infos_from_episodes(episodes).await
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
