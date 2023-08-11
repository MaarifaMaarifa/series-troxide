use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::tv_schedule::{get_episodes_with_country, get_episodes_with_date};

/// Retrieves series aired on a specific date through the provided optional &str
/// If None is supplied, it will default the the current day
pub async fn get_series_with_date(
    date: Option<&str>,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_date(date).await?;

    let handles: Vec<_> = episodes
        .into_iter()
        .map(|episode| {
            tokio::spawn(super::series_information::get_series_main_info_with_url(
                episode.links.show.href,
            ))
        })
        .collect();

    let mut series_information_strs = Vec::with_capacity(handles.len());
    for handle in handles {
        series_information_strs.push(handle.await??)
    }
    Ok(series_information_strs)
}

/// Retrieves series aired on the current day at a particular country provided in ISO 3166-1
pub async fn get_series_with_country(
    country_iso: &str,
) -> anyhow::Result<Vec<SeriesMainInformation>> {
    let episodes = get_episodes_with_country(country_iso).await?;

    let handles: Vec<_> = episodes
        .into_iter()
        .map(|episode| {
            tokio::spawn(super::series_information::get_series_main_info_with_url(
                episode.links.show.href,
            ))
        })
        .collect();

    let mut series_information_strs = Vec::with_capacity(handles.len());
    for handle in handles {
        series_information_strs.push(handle.await??)
    }
    Ok(series_information_strs)
}
