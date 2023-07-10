use super::{Series, SeriesStatus};
use crate::core::api::series_information::SeriesMainInformation;
use crate::gui::helpers::season_episode_str_gen;
use crate::gui::troxide_widget::{GREEN_THEME, INFO_HEADER, RED_THEME};

use iced::widget::{column, text};
use iced::Renderer;
use iced_aw::Grid;

use super::Message;

pub fn status_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    let title_text = text("Status");
    series_data_grid.insert(title_text);

    let status_text = match SeriesStatus::new(series_info) {
        SeriesStatus::Running => text("Running").style(GREEN_THEME),
        SeriesStatus::Ended => text("Ended").style(RED_THEME),
        SeriesStatus::ToBeDetermined => text("To Be Determined"),
        SeriesStatus::InDevelopment => text("In Development"),
        SeriesStatus::Other => text(&series_info.status),
    };
    series_data_grid.insert(status_text);
}

pub fn average_runtime_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    // since the the title part of this widget is the longest, we gonna add some space
    // infront of it to make the separation of column nicer
    series_data_grid.insert(text("Average runtime    "));
    let body_widget = if let Some(average_runtime) = series_info.average_runtime {
        text(format!("{} mins", average_runtime))
    } else {
        text("unavailable")
    };
    series_data_grid.insert(body_widget)
}

pub fn genres_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    if !series_info.genres.is_empty() {
        series_data_grid.insert(text("Genres"));
        let mut genres = String::new();

        let mut series_result_iter = series_info.genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            genres.push_str(genre);
            if series_result_iter.peek().is_some() {
                genres.push_str(", ");
            }
        }
        series_data_grid.insert(text(genres));
    }
}

pub fn language_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    series_data_grid.insert(text("Language"));
    series_data_grid.insert(if let Some(language) = &series_info.language {
        text(language)
    } else {
        text("unavailable")
    });
}

pub fn premiered_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    series_data_grid.insert(text("Premiered"));
    let body_text = if let Some(premier) = &series_info.premiered {
        text(premier)
    } else {
        text("unavailable")
    };

    series_data_grid.insert(body_text)
}

pub fn ended_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    // Pushing the widget to the grid only when the series has ended
    match SeriesStatus::new(series_info) {
        SeriesStatus::Ended => {}
        _ => return,
    }

    series_data_grid.insert(text("Ended"));
    let body_text = if let Some(ended) = &series_info.ended {
        text(ended)
    } else {
        text("unavailable")
    };

    series_data_grid.insert(body_text)
}

pub fn summary_widget(series_info: &SeriesMainInformation) -> iced::Element<'_, Message, Renderer> {
    if let Some(summary) = &series_info.summary {
        let summary = html2text::from_read(summary.as_bytes(), 1000);
        column![text("Summary"), text(summary).size(15),]
            .spacing(5)
            .into()
    } else {
        text("").into()
    }
}

pub fn rating_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    series_data_grid.insert(text("Average rating"));
    let body_wiget = if let Some(average_rating) = series_info.rating.average {
        text(average_rating.to_string())
    } else {
        text("unavailable")
    };

    series_data_grid.insert(body_wiget)
}

pub fn network_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    if let Some(network) = &series_info.network {
        // TODO: Add a clickable link
        series_data_grid.insert(text("Network"));
        series_data_grid.insert(text(format!(
            "{} ({})",
            &network.name, &network.country.name
        )))
    }
}

pub fn webchannel_widget(
    series_data_grid: &mut Grid<'_, Message, Renderer>,
    series_info: &SeriesMainInformation,
) {
    if let Some(webchannel) = &series_info.web_channel {
        // TODO: Add a clickable link
        series_data_grid.insert(text("Webchannel"));
        series_data_grid.insert(text(&webchannel.name))
    }
}

pub fn next_episode_release_time_widget(series: &Series) -> iced::widget::Text<'_, Renderer> {
    if let Some((episode, release_time)) = series.next_episode_release_time.as_ref() {
        let season = episode.season;
        let episode = episode.number.expect("Could not get episode number");

        let next_episode = season_episode_str_gen(season, episode);

        text(format!(
            "{} in {}",
            next_episode,
            release_time.get_remaining_release_time().unwrap()
        ))
        .size(INFO_HEADER)
    } else {
        text("")
    }
}
