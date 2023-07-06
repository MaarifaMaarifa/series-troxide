use iced::widget::{column, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length, Renderer};

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::database;

use super::Message;

pub fn watch_count() -> Element<'static, Message, Renderer> {
    let series_total_number = database::DB.get_total_series();
    let seasons_total_number = database::DB.get_total_seasons();
    let episodes_total_number = database::DB.get_total_episodes();

    let episodes_count = column![
        text(episodes_total_number).size(35),
        text("Episodes").size(15),
    ]
    .align_items(Alignment::Center);

    let series_seasons_count = row![
        column![text(series_total_number).size(35), text("Series").size(15)]
            .align_items(Alignment::Center),
        horizontal_space(10),
        column![
            text(seasons_total_number).size(35),
            text("Seasons").size(15)
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
        .into()
}

pub fn time_count(
    series_infos_and_time: &Vec<(SeriesMainInformation, u32)>,
) -> Element<'_, Message, Renderer> {
    let total_average_minutes: u32 = series_infos_and_time
        .into_iter()
        .map(|(_, average_runtime)| average_runtime)
        .sum();

    let total_minutes_count = column![
        text(total_average_minutes).size(35),
        text("Minutes").size(15)
    ]
    .align_items(Alignment::Center);

    let years = total_average_minutes / (60 * 24 * 365);
    let months = (total_average_minutes / (60 * 24 * 30)) % 12;
    let days = (total_average_minutes / (60 * 24)) % 30;
    let hours = (total_average_minutes / 60) % 24;

    let complete_time_count = row![
        column![text(years).size(35), text("Years").size(15)].align_items(Alignment::Center),
        column![text(months).size(35), text("Months").size(15)].align_items(Alignment::Center),
        column![text(days).size(35), text("Days").size(15)].align_items(Alignment::Center),
        column![text(hours).size(35), text("Hours").size(15)].align_items(Alignment::Center),
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
        .into()
}
