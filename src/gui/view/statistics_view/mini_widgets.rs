use iced::widget::{column, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length, Renderer};

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
    .align_items(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .padding(10)
        .center_x()
        .center_y()
        .into()
}
