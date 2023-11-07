use iced::widget::{column, container, horizontal_space, row, scrollable, text, Row, Space};
use iced::{Alignment, Element, Length, Renderer};
use iced_aw::Grid;

use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::database;
use crate::gui::{helpers, styles};

use super::Message;

pub fn watch_count() -> Element<'static, Message, Renderer> {
    let series_total_number = database::DB.get_total_series();
    let seasons_total_number = database::DB.get_total_seasons();
    let episodes_total_number = database::DB.get_total_episodes();

    let episodes_count = column![
        text(episodes_total_number)
            .size(31)
            .style(styles::text_styles::accent_color_theme()),
        text("Episodes").size(11),
    ]
    .align_items(Alignment::Center);

    let series_seasons_count = row![
        column![
            text(series_total_number)
                .size(31)
                .style(styles::text_styles::accent_color_theme()),
            text("Series").size(11)
        ]
        .align_items(Alignment::Center),
        horizontal_space(10),
        column![
            text(seasons_total_number)
                .size(31)
                .style(styles::text_styles::accent_color_theme()),
            text("Seasons").size(11)
        ]
        .align_items(Alignment::Center)
    ]
    .align_items(Alignment::Center);

    let content = column![
        text("You've seen a total of"),
        episodes_count,
        text("In exactly"),
        series_seasons_count,
    ]
    .align_items(Alignment::Center)
    .spacing(5);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .center_x()
        .center_y()
        .style(styles::container_styles::first_class_container_rounded_theme())
        .into()
}

pub fn time_count(
    series_infos_and_time: &[(SeriesMainInformation, Option<u32>)],
) -> Element<'_, Message, Renderer> {
    let total_average_minutes: u32 = series_infos_and_time
        .iter()
        .map(|(_, average_runtime)| average_runtime.unwrap_or(0))
        .sum();

    let total_minutes_count = column![
        text(total_average_minutes)
            .style(styles::text_styles::accent_color_theme())
            .size(31),
        text("Minutes").size(11)
    ]
    .align_items(Alignment::Center);

    let times = helpers::time::SaneTime::new(total_average_minutes).get_time_plurized();

    let complete_time_count: Element<'_, Message, Renderer> = if times.is_empty() {
        Space::new(0, 0).into()
    } else {
        let time_values: Vec<_> = times
            .into_iter()
            .rev()
            .map(|(time_text, time_value)| {
                column![
                    text(time_value)
                        .size(31)
                        .style(styles::text_styles::accent_color_theme()),
                    text(time_text).size(11)
                ]
                .align_items(Alignment::Center)
                .into()
            })
            .collect();

        let time_row = Row::with_children(time_values)
            .align_items(Alignment::Center)
            .spacing(10);

        column![text("Which is exactly"), time_row]
            .spacing(5)
            .align_items(Alignment::Center)
            .into()
    };

    let content = column![
        text("Total average time spent watching Series"),
        total_minutes_count,
        complete_time_count,
    ]
    .align_items(Alignment::Center)
    .spacing(5);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .center_x()
        .center_y()
        .style(styles::container_styles::first_class_container_rounded_theme())
        .into()
}

pub fn genre_stats(series_infos: Vec<&SeriesMainInformation>) -> Element<'_, Message, Renderer> {
    use crate::core::api::tv_maze::series_information::Genre;
    use std::collections::HashMap;

    if series_infos.is_empty() {
        return Space::new(0, 0).into();
    }

    let mut genre_count: HashMap<Genre, usize> = HashMap::new();

    series_infos.iter().for_each(|series_info| {
        series_info.get_genres().into_iter().for_each(|genre| {
            genre_count
                .entry(genre)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        })
    });

    let mut genre_count: Vec<(Genre, usize)> = genre_count
        .into_iter()
        .map(|(genre, count)| (genre, count))
        .collect();

    genre_count.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    let mut content = Grid::with_columns(2);

    genre_count.into_iter().for_each(|(genre, count)| {
        content.insert(
            text(format!("{}    ", genre)).style(styles::text_styles::accent_color_theme()),
        );
        content.insert(text(format!("{} series", count)));
    });

    let content = column![text("Genre Stats"), content]
        .align_items(Alignment::Center)
        .spacing(10)
        .width(Length::Fill)
        .padding(10);

    let content = scrollable(content)
        .direction(styles::scrollable_styles::vertical_direction())
        .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .style(styles::container_styles::first_class_container_rounded_theme())
        .into()
}

pub mod series_banner {
    use std::sync::mpsc;

    use iced::widget::{column, container, image, mouse_area, row, text, Row};
    use iced::{Alignment, Command, Element, Length, Renderer};

    use crate::core::{api::tv_maze::series_information::SeriesMainInformation, database};
    pub use crate::gui::message::IndexedMessage;
    use crate::gui::troxide_widget::series_poster::{GenericPoster, GenericPosterMessage};
    use crate::gui::{helpers, styles};

    #[derive(Debug, Clone)]
    pub enum Message {
        Poster(GenericPosterMessage),
        Selected,
    }

    pub struct SeriesBanner<'a> {
        index: usize,
        poster: GenericPoster<'a>,
        watch_time: Option<u32>,
    }

    impl<'a> SeriesBanner<'a> {
        pub fn new(
            index: usize,
            series_info: std::borrow::Cow<'a, SeriesMainInformation>,
            watch_time: Option<u32>,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (Self, Command<IndexedMessage<Message>>) {
            let (poster, poster_command) = GenericPoster::new(series_info, series_page_sender);
            (
                Self {
                    index,
                    poster,
                    watch_time,
                },
                poster_command
                    .map(Message::Poster)
                    .map(move |message| IndexedMessage::new(index, message)),
            )
        }

        pub fn update(&mut self, message: IndexedMessage<Message>) {
            match message.message() {
                Message::Selected => self.poster.open_series_page(),
                Message::Poster(message) => self.poster.update(message),
            }
        }

        pub fn view(&self) -> Element<'_, IndexedMessage<Message>, Renderer> {
            let series_id = self.poster.get_series_info().id;
            let series = database::DB.get_series(series_id).unwrap();

            let series_name = format!(
                "{}: {}",
                self.index + 1,
                &self.poster.get_series_info().name
            );
            let times = self
                .watch_time
                .map(|time| helpers::time::SaneTime::new(time).get_time_plurized())
                .unwrap_or_default();

            let seasons = series.get_total_seasons();
            let episodes = series.get_total_episodes();

            let time_stats = Row::with_children(
                times
                    .into_iter()
                    .rev()
                    .map(|(time_text, time_value)| {
                        column![
                            text(time_value)
                                .size(20)
                                .style(styles::text_styles::accent_color_theme()),
                            text(time_text).size(11)
                        ]
                        .align_items(Alignment::Center)
                        .spacing(5)
                        .into()
                    })
                    .collect(),
            )
            .align_items(Alignment::Center)
            .spacing(5);

            let count_stats = row![
                column![
                    text(seasons)
                        .size(20)
                        .style(styles::text_styles::accent_color_theme()),
                    text("Seasons").size(11)
                ]
                .align_items(Alignment::Center),
                column![
                    text(episodes)
                        .size(20)
                        .style(styles::text_styles::accent_color_theme()),
                    text("episodes").size(11)
                ]
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center)
            .spacing(5);

            let metadata = column![count_stats, time_stats]
                .align_items(Alignment::Center)
                .spacing(5);

            let banner: Element<'_, Message, Renderer> =
                if let Some(image_bytes) = self.poster.get_image() {
                    let image_handle = image::Handle::from_memory(image_bytes.clone());
                    image(image_handle).height(100).into()
                } else {
                    helpers::empty_image::empty_image()
                        .width(71)
                        .height(100)
                        .into()
                };

            let content = column![text(series_name), metadata]
                .spacing(5)
                .align_items(Alignment::Center);

            let content = row![banner, container(content).center_x().width(Length::Fill)]
                .spacing(5)
                .padding(5)
                .width(Length::Fill);

            let element: Element<'_, Message, Renderer> = mouse_area(
                container(content)
                    .width(300)
                    .style(styles::container_styles::first_class_container_rounded_theme())
                    .padding(10)
                    .center_x()
                    .center_y(),
            )
            .on_press(Message::Selected)
            .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }
    }
}
