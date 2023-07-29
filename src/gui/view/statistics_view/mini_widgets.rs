use iced::widget::{column, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length, Renderer};

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::database;
use crate::gui::styles;

use super::Message;

pub fn watch_count() -> Element<'static, Message, Renderer> {
    let series_total_number = database::DB.get_total_series();
    let seasons_total_number = database::DB.get_total_seasons();
    let episodes_total_number = database::DB.get_total_episodes();

    let episodes_count = column![
        text(episodes_total_number).size(31),
        text("Episodes").size(11),
    ]
    .align_items(Alignment::Center);

    let series_seasons_count = row![
        column![text(series_total_number).size(31), text("Series").size(11)]
            .align_items(Alignment::Center),
        horizontal_space(10),
        column![
            text(seasons_total_number).size(31),
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
    series_infos_and_time: &[(SeriesMainInformation, u32)],
) -> Element<'_, Message, Renderer> {
    let total_average_minutes: u32 = series_infos_and_time
        .iter()
        .map(|(_, average_runtime)| average_runtime)
        .sum();

    let total_minutes_count = column![
        text(total_average_minutes).size(31),
        text("Minutes").size(11)
    ]
    .align_items(Alignment::Center);

    let years = total_average_minutes / (60 * 24 * 365);
    let months = (total_average_minutes / (60 * 24 * 30)) % 12;
    let days = (total_average_minutes / (60 * 24)) % 30;
    let hours = (total_average_minutes / 60) % 24;

    let complete_time_count = row![
        column![text(years).size(31), text("Years").size(11)].align_items(Alignment::Center),
        column![text(months).size(31), text("Months").size(11)].align_items(Alignment::Center),
        column![text(days).size(31), text("Days").size(11)].align_items(Alignment::Center),
        column![text(hours).size(31), text("Hours").size(11)].align_items(Alignment::Center),
    ]
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
    use iced::widget::{column, container, row, text};
    use iced::{Alignment, Command, Element, Length, Renderer};

    // use crate::core::caching;
    use crate::core::{api::series_information::SeriesMainInformation, database};

    #[derive(Debug, Clone)]
    pub enum Message {
        #[allow(dead_code)]
        BannerReceived(usize, Option<Vec<u8>>),
    }

    impl Message {
        pub fn get_id(&self) -> usize {
            match self {
                Message::BannerReceived(id, _) => *id,
            }
        }
    }

    pub struct SeriesBanner {
        series_info_and_time: (SeriesMainInformation, u32),
        banner: Option<Vec<u8>>,
    }

    impl SeriesBanner {
        pub fn new(
            _id: usize,
            series_info_and_time: (SeriesMainInformation, u32),
        ) -> (Self, Command<Message>) {
            let _series_id = series_info_and_time.0.id;
            (
                Self {
                    series_info_and_time,
                    banner: None,
                },
                // TODO: Request show banner
                // for this to not cause problem when rendering, a better way of rendering
                // the banners list hould be implemented as iced runs out of memory an panics
                /*Command::perform(
                    caching::show_images::get_recent_banner(series_id),
                    move |message| Message::BannerReceived(id, message),
                ),*/
                Command::none(),
            )
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::BannerReceived(_, banner) => self.banner = banner,
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let series_id = self.series_info_and_time.0.id;
            let series = database::DB.get_series(series_id).unwrap();

            let series_name = &self.series_info_and_time.0.name;
            let time_in_hours = self.series_info_and_time.1 / 60;
            let seasons = series.get_total_seasons();
            let episodes = series.get_total_episodes();

            let metadata = row![
                column![text(time_in_hours).size(31), text("Hours").size(11)]
                    .align_items(Alignment::Center),
                column![text(seasons).size(31), text("Seasons").size(11)]
                    .align_items(Alignment::Center),
                column![text(episodes).size(31), text("episodes").size(11)]
                    .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center)
            .spacing(5);

            // let banner: Element<'_, Message, Renderer> =
            //     if let Some(image_bytes) = self.banner.clone() {
            //         let image_handle = image::Handle::from_memory(image_bytes);
            //         image(image_handle).height(100).into()
            //     } else {
            //         container("").into()
            //     };

            let content = column![/*banner,*/ text(series_name), metadata]
                .spacing(5)
                .align_items(Alignment::Center);

            container(content)
                .width(Length::Fill)
                .padding(10)
                .center_x()
                .center_y()
                .into()
        }
    }
}
