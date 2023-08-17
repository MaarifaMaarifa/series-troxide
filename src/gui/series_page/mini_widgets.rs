use super::SeriesStatus;
use crate::core::api::episodes_information::Episode;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::gui::assets::get_static_cow_from_asset;
use crate::gui::assets::icons::{CLOCK_FILL, STAR, STAR_FILL, STAR_HALF};
use crate::gui::helpers::{self, season_episode_str_gen};
use crate::gui::styles;

use iced::widget::{container, horizontal_space, row, svg, text, Space};
use iced::{Element, Length, Renderer};

use super::Message;

pub fn status_widget(
    series_info: &SeriesMainInformation,
) -> (
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
) {
    let title_text = text("Status");

    let status_text = match SeriesStatus::new(series_info) {
        SeriesStatus::Running => text("Running").style(styles::text_styles::green_text_theme()),
        SeriesStatus::Ended => text("Ended").style(styles::text_styles::red_text_theme()),
        SeriesStatus::ToBeDetermined => text("To Be Determined"),
        SeriesStatus::InDevelopment => text("In Development"),
        SeriesStatus::Other => text(&series_info.status),
    };
    (title_text.into(), status_text.into())
}

pub fn average_runtime_widget(
    series_info: &SeriesMainInformation,
) -> (
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
) {
    // since the the title part of this widget is the longest, we gonna add some space
    // infront of it to make the separation of column nicer
    let title_text = text("Average runtime    ");
    let body_widget = if let Some(average_runtime) = series_info.average_runtime {
        text(format!("{} mins", average_runtime))
    } else {
        text("unavailable")
    };
    (title_text.into(), body_widget.into())
}

pub fn genres_widget(
    series_info: &SeriesMainInformation,
) -> Option<(
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
)> {
    if !series_info.genres.is_empty() {
        let title_text = text("Genres");
        let mut genres = String::new();

        let mut series_result_iter = series_info.genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            genres.push_str(genre);
            if series_result_iter.peek().is_some() {
                genres.push_str(" | ");
            }
        }
        let genres = text(genres);

        Some((title_text.into(), genres.into()))
    } else {
        None
    }
}

pub fn language_widget(
    series_info: &SeriesMainInformation,
) -> (
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
) {
    let title_text = text("Language");
    let language = if let Some(language) = &series_info.language {
        text(language)
    } else {
        text("unavailable")
    };

    (title_text.into(), language.into())
}

pub fn premiered_widget(
    series_info: &SeriesMainInformation,
) -> (
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
) {
    let title_text = text("Premiered");
    let body_text = if let Some(premier) = &series_info.premiered {
        text(premier)
    } else {
        text("unavailable")
    };

    (title_text.into(), body_text.into())
}

pub fn ended_widget(
    series_info: &SeriesMainInformation,
) -> Option<(
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
)> {
    // Pushing the widget to the grid only when the series has ended
    match SeriesStatus::new(series_info) {
        SeriesStatus::Ended => {}
        _ => return None,
    }

    let title_text = text("Ended");
    let body_text = if let Some(ended) = &series_info.ended {
        text(ended)
    } else {
        text("unavailable")
    };

    Some((title_text.into(), body_text.into()))
}

pub fn summary_widget(series_info: &SeriesMainInformation) -> iced::Element<'_, Message, Renderer> {
    if let Some(summary) = &series_info.summary {
        let summary = html2text::from_read(summary.as_bytes(), 1000);
        text(summary).size(11).width(880).into()
    } else {
        text("").into()
    }
}

pub fn rating_widget(series_info: &SeriesMainInformation) -> Element<'_, Message, Renderer> {
    if let Some(average_rating) = series_info.rating.average {
        let star_handle = svg::Handle::from_memory(get_static_cow_from_asset(STAR));
        let star_half_handle = svg::Handle::from_memory(get_static_cow_from_asset(STAR_HALF));
        let star_fill_handle = svg::Handle::from_memory(get_static_cow_from_asset(STAR_FILL));

        let mut rating = row![];

        let total_rating = 10_u8;
        let series_rating = average_rating as u8;
        let mut missing_rating = total_rating - series_rating;

        let rating_text = text(format!("{} / {}", average_rating, total_rating));

        for _ in 0..series_rating {
            rating = rating.push(
                svg(star_fill_handle.clone())
                    .width(15)
                    .height(15)
                    .style(styles::svg_styles::colored_svg_theme()),
            )
        }

        if average_rating.trunc() != average_rating {
            missing_rating -= 1;
            rating = rating.push(
                svg(star_half_handle)
                    .width(15)
                    .height(15)
                    .style(styles::svg_styles::colored_svg_theme()),
            )
        };

        for _ in 0..missing_rating {
            rating = rating.push(
                svg(star_handle.clone())
                    .width(15)
                    .height(15)
                    .style(styles::svg_styles::colored_svg_theme()),
            )
        }

        rating = rating.push(horizontal_space(10));
        rating = rating.push(rating_text);

        rating.into()
    } else {
        Space::new(0, 0).into()
    }
}

pub fn network_widget(
    series_info: &SeriesMainInformation,
) -> Option<(
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
)> {
    series_info.network.as_ref().map(|network| {
        // TODO: Add a clickable link
        (
            text("Network").into(),
            text(format!("{} ({})", &network.name, &network.country.name)).into(),
        )
    })
}

pub fn webchannel_widget(
    series_info: &SeriesMainInformation,
) -> Option<(
    Element<'_, Message, Renderer>,
    Element<'_, Message, Renderer>,
)> {
    series_info.web_channel.as_ref().map(|webchannel| {
        // TODO: Add a clickable link
        (text("Webchannel").into(), text(&webchannel.name).into())
    })
}

pub fn next_episode_release_time_widget(
    next_episode_release_time: Option<&(Episode, EpisodeReleaseTime)>,
) -> Element<'_, Message, Renderer> {
    if let Some((episode, release_time)) = next_episode_release_time {
        let season = episode.season;
        let episode = episode.number.expect("Could not get episode number");

        let next_episode = season_episode_str_gen(season, episode);
        let clock_icon_handle = svg::Handle::from_memory(get_static_cow_from_asset(CLOCK_FILL));
        let clock_icon = svg(clock_icon_handle)
            .width(Length::Shrink)
            .style(styles::svg_styles::colored_svg_theme());

        let text = text(format!(
            "{} in {}",
            next_episode,
            helpers::time::SaneTime::new(
                release_time.get_remaining_release_duration().num_minutes() as u32
            )
            .get_time()
            .into_iter()
            .rev()
            .map(|(time_text, time_value)| format!("{} {} ", time_value, time_text))
            .collect::<String>()
        ))
        .size(14);

        container(row![clock_icon, text].spacing(5))
            .style(styles::container_styles::second_class_container_square_theme())
            .padding(5)
            .into()
    } else {
        Space::new(0, 0).into()
    }
}
