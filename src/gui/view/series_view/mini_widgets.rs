use super::{Series, SeriesStatus};
use crate::core::api::series_information::SeriesMainInformation;
use crate::gui::helpers::parse_season_episode_number;
use crate::gui::troxide_widget::{GREEN_THEME, INFO_BODY, INFO_HEADER, RED_THEME};

use iced::alignment;
use iced::widget::{row, text};
use iced::Renderer;

use super::Message;

pub fn status_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Status: ").size(INFO_HEADER));

    let status_text = match SeriesStatus::new(series_info) {
        SeriesStatus::Running => text("Running").style(GREEN_THEME),
        SeriesStatus::Ended => text("Ended").style(RED_THEME),
        SeriesStatus::ToBeDetermined => text("To Be Determined"),
        SeriesStatus::InDevelopment => text("In Development"),
        SeriesStatus::Other => text(&series_info.status),
    }
    .vertical_alignment(alignment::Vertical::Bottom)
    .size(INFO_BODY)
    .height(INFO_HEADER);

    row.push(status_text)
}

pub fn average_runtime_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Average runtime: ").size(INFO_HEADER));
    let body_widget = if let Some(average_runtime) = series_info.average_runtime {
        text(format!("{} mins", average_runtime))
    } else {
        text("unavailable")
    };
    row.push(
        body_widget
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

pub fn genres_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    if !series_info.genres.is_empty() {
        let row = row!(text("Genres: ").size(INFO_HEADER));
        let mut genres = String::new();

        let mut series_result_iter = series_info.genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            genres.push_str(genre);
            if series_result_iter.peek().is_some() {
                genres.push_str(", ");
            }
        }
        row.push(text(genres).size(INFO_BODY))
    } else {
        row!()
    }
}

pub fn language_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(
        text("Language: ").size(INFO_HEADER),
        if let Some(language) = &series_info.language {
            text(language)
        } else {
            text("unavailable")
        }
        .size(INFO_BODY)
        .height(INFO_HEADER)
        .vertical_alignment(alignment::Vertical::Bottom)
    );
    row
}

pub fn premiered_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Premiered: ").size(INFO_HEADER));
    let body_text = if let Some(premier) = &series_info.premiered {
        text(premier)
    } else {
        text("unavailable")
    };

    row.push(
        body_text
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

pub fn ended_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    // Creating the widget only when the series has ended
    match SeriesStatus::new(series_info) {
        SeriesStatus::Ended => {}
        _ => return row!(),
    }

    let row = row!(text("Ended: ").size(INFO_HEADER));
    let body_text = if let Some(ended) = &series_info.ended {
        text(ended)
    } else {
        text("unavailable")
    };

    row.push(
        body_text
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

pub fn summary_widget(series_info: &SeriesMainInformation) -> iced::widget::Text<'_, Renderer> {
    if let Some(summary) = &series_info.summary {
        text(summary).size(15)
    } else {
        text("")
    }
}

pub fn rating_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Average rating: ").size(INFO_HEADER));
    let body_wiget = if let Some(average_rating) = series_info.rating.average {
        text(average_rating.to_string())
    } else {
        text("unavailable")
    };

    row.push(
        body_wiget
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

pub fn network_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    if let Some(network) = &series_info.network {
        // TODO: Add a clickable link
        row!(
            text("Network:  ").size(INFO_HEADER),
            text(format!("{} ({})", &network.name, &network.country.name))
                .size(INFO_BODY)
                .height(INFO_HEADER)
                .vertical_alignment(alignment::Vertical::Bottom),
        )
    } else {
        row!()
    }
}

pub fn webchannel_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    if let Some(webchannel) = &series_info.web_channel {
        // TODO: Add a clickable link
        row!(
            text("Webchannel: ").size(INFO_HEADER),
            text(&webchannel.name)
                .size(INFO_BODY)
                .height(INFO_HEADER)
                .vertical_alignment(alignment::Vertical::Bottom),
        )
    } else {
        row!()
    }
}

pub fn next_episode_release_time_widget(series: &Series) -> iced::widget::Text<'_, Renderer> {
    if let Some((episode, release_time)) = series.next_episode_release_time.as_ref() {
        let season = episode.season;
        let episode = episode.number.expect("Could not get episode number");

        let next_episode = format!(
            "S{}E{}",
            parse_season_episode_number(season),
            parse_season_episode_number(episode)
        );

        text(format!(
            "Next episode: {} release in {}",
            next_episode, release_time
        ))
        .size(INFO_HEADER)
    } else {
        text("")
    }
}
