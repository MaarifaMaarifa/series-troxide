use bytes::Bytes;

use super::Message;
use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::{SeriesMainInformation, ShowStatus};
use crate::core::database;
use crate::gui::assets::icons::{
    CLOCK_FILL, PATCH_PLUS, PATCH_PLUS_FILL, STAR, STAR_FILL, STAR_HALF,
};
use crate::gui::helpers::{self, season_episode_str_gen};
use crate::gui::styles;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, svg, text, Button, Space,
};
use iced::{Alignment, Element, Length};
use iced_aw::{Grid, GridRow};

/// Generates the Series Metadata
pub fn series_metadata<'a>(
    series_information: &'a SeriesMainInformation,
    image_bytes: Option<Bytes>,
    next_episode_to_air: Option<&'a Episode>,
) -> Element<'a, Message> {
    let mut main_info = row!().padding(5).spacing(10);

    if let Some(image_bytes) = image_bytes {
        let image_handle = iced::widget::image::Handle::from_memory(image_bytes);
        let image = iced::widget::image(image_handle).width(180);

        main_info = main_info.push(image);
    } else {
        main_info = main_info.push(helpers::empty_image::empty_image().width(180).height(253));
    };

    let mut data_grid = Grid::new();

    data_grid = status_widget(series_information, data_grid);
    data_grid = series_type_widget(series_information, data_grid);
    data_grid = genres_widget(series_information, data_grid);
    data_grid = language_widget(series_information, data_grid);
    data_grid = average_runtime_widget(series_information, data_grid);
    data_grid = network_widget(series_information, data_grid);
    data_grid = webchannel_widget(series_information, data_grid);
    data_grid = premiered_widget(series_information, data_grid);
    data_grid = ended_widget(series_information, data_grid);

    let rating_widget = rating_widget(series_information);
    let summary = summary_widget(series_information);

    let series_name = text(series_information.name.clone())
        .size(31)
        .style(styles::text_styles::accent_color_theme());

    let title_bar = row![
        series_name.width(Length::FillPortion(10)),
        tracking_button(series_information.id)
    ];

    let next_episode_widget = next_episode_to_air_widget(next_episode_to_air);

    let rating_and_release_widget =
        row![rating_widget, horizontal_space(), next_episode_widget].padding(3);

    let series_data = column![
        title_bar,
        rating_and_release_widget,
        horizontal_rule(1),
        data_grid,
        Space::with_height(10),
    ]
    .width(700)
    .spacing(5);

    main_info = main_info.push(series_data);

    let content = container(
        column![main_info, summary]
            .align_items(Alignment::Center)
            .padding(5),
    )
    .style(styles::container_styles::first_class_container_square_theme());

    container(content)
        .width(Length::Fill)
        .padding(10)
        .center_x()
        .into()
}

pub fn background(
    background_bytes: Option<Bytes>,
    series_image_blurred: Option<image::DynamicImage>,
) -> Element<'static, Message> {
    if let Some(image_bytes) = background_bytes {
        let image_handle = iced::widget::image::Handle::from_memory(image_bytes);
        iced::widget::image(image_handle)
            .width(Length::Fill)
            .height(300)
            .content_fit(iced::ContentFit::Cover)
            .into()
    } else {
        // using the blurred series image when the background is not yet present(or still loading)
        if let Some(image) = series_image_blurred {
            let image_handle = iced::widget::image::Handle::from_pixels(
                image.width(),
                image.height(),
                image.into_rgba8().into_vec(),
            );
            return iced::widget::image(image_handle)
                .width(Length::Fill)
                .height(300)
                .content_fit(iced::ContentFit::Cover)
                .into();
        }
        Space::new(0, 300).into()
    }
}

pub fn tracking_button(series_id: u32) -> Button<'static, Message> {
    if database::DB
        .get_series(series_id)
        .map(|series| series.is_tracked())
        .unwrap_or(false)
    {
        let tracked_icon_handle = svg::Handle::from_memory(PATCH_PLUS_FILL);
        let icon = svg(tracked_icon_handle)
            .width(30)
            .height(30)
            .style(styles::svg_styles::colored_svg_theme());
        button(icon).on_press(Message::UntrackSeries)
    } else {
        let tracked_icon_handle = svg::Handle::from_memory(PATCH_PLUS);
        let icon = svg(tracked_icon_handle)
            .width(30)
            .height(30)
            .style(styles::svg_styles::colored_svg_theme());
        button(icon).on_press(Message::TrackSeries)
    }
    .style(styles::button_styles::transparent_button_theme())
}

pub fn status_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    let series_status = series_info.get_status();

    let mut status_text = text(&series_status);

    if let ShowStatus::Running = series_status {
        status_text = status_text.style(styles::text_styles::green_text_theme())
    }
    if let ShowStatus::Ended = series_status {
        status_text = status_text.style(styles::text_styles::red_text_theme())
    }

    data_grid.push(GridRow::new().push(text("Status")).push(status_text))
}

pub fn series_type_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    if let Some(kind) = series_info.kind.as_ref() {
        return data_grid.push(GridRow::new().push(text("Type")).push(text(kind)));
    };
    data_grid
}

pub fn average_runtime_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    // since the the title part of this widget is the longest, we gonna add some space
    // infront of it to make the separation of column nicer
    let title_text = text("Average runtime    ");
    let body_widget = if let Some(average_runtime) = series_info.average_runtime {
        text(format!("{} mins", average_runtime))
    } else {
        text("unavailable")
    };

    data_grid.push(GridRow::new().push(title_text).push(body_widget))
}

pub fn genres_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    if !series_info.genres.is_empty() {
        let title_text = text("Genres");
        let genres = text(helpers::genres_with_pipes(&series_info.genres));

        return data_grid.push(GridRow::new().push(title_text).push(genres));
    }

    data_grid
}

pub fn language_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    let title_text = text("Language");
    let language = if let Some(language) = &series_info.language {
        text(language)
    } else {
        text("unavailable")
    };

    data_grid.push(GridRow::new().push(title_text).push(language))
}

pub fn premiered_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    let title_text = text("Premiered");
    let body_text = if let Some(premier) = &series_info.premiered {
        text(premier)
    } else {
        text("unavailable")
    };

    data_grid.push(GridRow::new().push(title_text).push(body_text))
}

pub fn ended_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    if let ShowStatus::Ended = series_info.get_status() {
        let title_text = text("Ended");
        let body_text = if let Some(ended) = &series_info.ended {
            text(ended)
        } else {
            text("unavailable")
        };

        return data_grid.push(GridRow::new().push(title_text).push(body_text));
    }
    data_grid
}

pub fn summary_widget(series_info: &SeriesMainInformation) -> iced::Element<'_, Message> {
    if let Some(summary) = &series_info.summary {
        let summary = html2text::from_read(summary.as_bytes(), 1000);
        text(summary).size(11).width(880).into()
    } else {
        text("").into()
    }
}

pub fn rating_widget(series_info: &SeriesMainInformation) -> Element<'_, Message> {
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

        rating = rating.push(Space::with_width(10));
        rating = rating.push(rating_text);

        rating.into()
    } else {
        Space::new(0, 0).into()
    }
}

pub fn network_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    if let Some(network) = series_info.network.as_ref() {
        if let Some(network_name) = network.country.name.as_ref() {
            // TODO: Add a clickable link
            return data_grid.push(
                GridRow::new()
                    .push(text("Network"))
                    .push(text(format!("{} ({})", &network.name, network_name))),
            );
        }
    };

    data_grid
}

pub fn webchannel_widget<'b>(
    series_info: &SeriesMainInformation,
    data_grid: Grid<'b, Message>,
) -> Grid<'b, Message> {
    if let Some(webchannel) = series_info.web_channel.as_ref() {
        // TODO: Add a clickable link
        return data_grid.push(
            GridRow::new()
                .push(text("Webchannel"))
                .push(text(&webchannel.name)),
        );
    };
    data_grid
}

pub fn next_episode_to_air_widget(next_episode_to_air: Option<&Episode>) -> Element<'_, Message> {
    if let Some((episode, Some(release_time))) =
        next_episode_to_air.map(|episode| (episode, episode.release_time().ok()))
    {
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
            helpers::time::NaiveTime::new(
                release_time.get_remaining_release_duration().num_minutes() as u32
            )
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
