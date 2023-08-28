use iced::widget::{column, container, horizontal_space, row, text, Row};
use iced::{Alignment, Element, Length, Renderer};

use crate::core::api::series_information::SeriesMainInformation;
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
            .style(styles::text_styles::purple_text_theme()),
        text("Episodes").size(11),
    ]
    .align_items(Alignment::Center);

    let series_seasons_count = row![
        column![
            text(series_total_number)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
            text("Series").size(11)
        ]
        .align_items(Alignment::Center),
        horizontal_space(10),
        column![
            text(seasons_total_number)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
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
            .style(styles::text_styles::purple_text_theme())
            .size(31),
        text("Minutes").size(11)
    ]
    .align_items(Alignment::Center);

    let times = helpers::time::SaneTime::new(total_average_minutes).get_time_plurized();

    let time_values: Vec<_> = times
        .into_iter()
        .rev()
        .map(|(time_text, time_value)| {
            column![
                text(time_value)
                    .size(31)
                    .style(styles::text_styles::purple_text_theme()),
                text(time_text).size(11)
            ]
            .align_items(Alignment::Center)
            .into()
        })
        .collect();

    let complete_time_count = Row::with_children(time_values)
        .align_items(Alignment::Center)
        .spacing(10);

    let content = column![
        text("Total average time spent watching Series"),
        total_minutes_count,
        text("Which is exactly"),
        complete_time_count,
    ]
    .align_items(Alignment::Center)
    .spacing(5);

    container(content)
        .width(Length::Fill)
        .padding(10)
        .center_x()
        .center_y()
        .style(styles::container_styles::first_class_container_rounded_theme())
        .into()
}

pub mod series_banner {
    use std::sync::mpsc;

    use bytes::Bytes;
    use iced::widget::{column, container, image, mouse_area, row, text, Row, Space};
    use iced::{Alignment, Command, Element, Length, Renderer};

    use crate::core::caching;
    use crate::core::{api::series_information::SeriesMainInformation, database};
    pub use crate::gui::message::IndexedMessage;
    use crate::gui::{helpers, styles};

    #[derive(Debug, Clone)]
    pub enum Message {
        #[allow(dead_code)]
        BannerReceived(Option<Bytes>),
        Selected,
    }

    pub struct SeriesBanner {
        index: usize,
        series_info_and_time: (SeriesMainInformation, Option<u32>),
        banner: Option<Bytes>,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    }

    impl SeriesBanner {
        pub fn new(
            index: usize,
            series_info_and_time: (SeriesMainInformation, Option<u32>),
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (Self, Command<IndexedMessage<Message>>) {
            let image_url = series_info_and_time
                .0
                .clone()
                .image
                .map(|image| image.medium_image_url);
            (
                Self {
                    index,
                    series_info_and_time,
                    banner: None,
                    series_page_sender,
                },
                image_url
                    .map(|image_url| {
                        Command::perform(caching::load_image(image_url), Message::BannerReceived)
                            .map(move |message| IndexedMessage::new(index, message))
                    })
                    .unwrap_or(Command::none()),
            )
        }

        pub fn update(&mut self, message: IndexedMessage<Message>) {
            match message.message() {
                Message::BannerReceived(banner) => self.banner = banner,
                Message::Selected => self
                    .series_page_sender
                    .send(self.series_info_and_time.0.clone())
                    .expect("failed to send series page"),
            }
        }

        pub fn view(&self) -> Element<'_, IndexedMessage<Message>, Renderer> {
            let series_id = self.series_info_and_time.0.id;
            let series = database::DB.get_series(series_id).unwrap();

            let series_name = format!("{}: {}", self.index + 1, &self.series_info_and_time.0.name);
            let times = self
                .series_info_and_time
                .1
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
                                .style(styles::text_styles::purple_text_theme()),
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
                        .style(styles::text_styles::purple_text_theme()),
                    text("Seasons").size(11)
                ]
                .align_items(Alignment::Center),
                column![
                    text(episodes)
                        .size(20)
                        .style(styles::text_styles::purple_text_theme()),
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
                if let Some(image_bytes) = self.banner.clone() {
                    let image_handle = image::Handle::from_memory(image_bytes);
                    image(image_handle).height(100).into()
                } else {
                    Space::new(71, 100).into()
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
