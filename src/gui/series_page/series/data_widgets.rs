use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::{SeriesMainInformation, ShowStatus};
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::gui::assets::icons::{CLOCK_FILL, STAR, STAR_FILL, STAR_HALF};
use crate::gui::helpers::{self, season_episode_str_gen};
use crate::gui::styles;

use iced::widget::{container, horizontal_space, row, svg, text, Space};
use iced::{Element, Length, Renderer};
use iced_aw::Grid;

use super::Message;

pub fn status_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    let series_status = series_info.get_status();

    let mut status_text = text(&series_status);

    if let ShowStatus::Running = series_status {
        status_text = status_text.style(styles::text_styles::green_text_theme())
    }
    if let ShowStatus::Ended = series_status {
        status_text = status_text.style(styles::text_styles::red_text_theme())
    }

    data_grid.insert(text("Status"));
    data_grid.insert(status_text);
}

pub fn series_type_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    if let Some(kind) = series_info.kind.as_ref() {
        data_grid.insert(text("Type"));
        data_grid.insert(text(kind));
    };
}

pub fn average_runtime_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    // since the the title part of this widget is the longest, we gonna add some space
    // infront of it to make the separation of column nicer
    let title_text = text("Average runtime    ");
    let body_widget = if let Some(average_runtime) = series_info.average_runtime {
        text(format!("{} mins", average_runtime))
    } else {
        text("unavailable")
    };

    data_grid.insert(title_text);
    data_grid.insert(body_widget);
}

pub fn genres_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    if !series_info.genres.is_empty() {
        let title_text = text("Genres");
        let genres = text(helpers::genres_with_pipes(&series_info.genres));

        data_grid.insert(title_text);
        data_grid.insert(genres);
    }
}

pub fn language_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    let title_text = text("Language");
    let language = if let Some(language) = &series_info.language {
        text(language)
    } else {
        text("unavailable")
    };

    data_grid.insert(title_text);
    data_grid.insert(language);
}

pub fn premiered_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    let title_text = text("Premiered");
    let body_text = if let Some(premier) = &series_info.premiered {
        text(premier)
    } else {
        text("unavailable")
    };

    data_grid.insert(title_text);
    data_grid.insert(body_text);
}

pub fn ended_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    if let ShowStatus::Ended = series_info.get_status() {
        let title_text = text("Ended");
        let body_text = if let Some(ended) = &series_info.ended {
            text(ended)
        } else {
            text("unavailable")
        };

        data_grid.insert(title_text);
        data_grid.insert(body_text);
    }
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
        let star_handle = svg::Handle::from_memory(STAR);
        let star_half_handle = svg::Handle::from_memory(STAR_HALF);
        let star_fill_handle = svg::Handle::from_memory(STAR_FILL);

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
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    if let Some(network) = series_info.network.as_ref() {
        // TODO: Add a clickable link
        data_grid.insert(text("Network"));
        data_grid.insert(text(format!(
            "{} ({})",
            &network.name, &network.country.name
        )));
    };
}

pub fn webchannel_widget(
    series_info: &SeriesMainInformation,
    data_grid: &mut Grid<'_, Message, Renderer>,
) {
    if let Some(webchannel) = series_info.web_channel.as_ref() {
        // TODO: Add a clickable link
        data_grid.insert(text("Webchannel"));
        data_grid.insert(text(&webchannel.name));
    };
}

pub fn next_episode_release_time_widget(
    next_episode_release_time: Option<&(Episode, EpisodeReleaseTime)>,
) -> Element<'_, Message, Renderer> {
    if let Some((episode, release_time)) = next_episode_release_time {
        let season = episode.season;
        let episode = episode.number.expect("Could not get episode number");

        let next_episode = season_episode_str_gen(season, episode);
        let clock_icon_handle = svg::Handle::from_memory(CLOCK_FILL);
        let clock_icon = svg(clock_icon_handle)
            .width(Length::Shrink)
            .style(styles::svg_styles::colored_svg_theme());

        let text = text(format!(
            "{} in {}",
            next_episode,
            helpers::time::SaneTime::new(
                release_time.get_remaining_release_duration().num_minutes() as u32
            )
            .get_time_plurized()
            .into_iter()
            .rev()
            .fold(String::new(), |acc, (time_text, time_value)| acc
                + &format!("{} {} ", time_value, time_text))
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
