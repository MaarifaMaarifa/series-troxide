use iced::widget::{column, container, scrollable};
use iced::{Command, Element, Length, Renderer};

use crate::{
    core::{api::series_information::SeriesMainInformation, database},
    gui::{Message as GuiMessage, Tab},
};
use mini_widgets::*;

mod mini_widgets;

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfosAndTimeReceived(Vec<(SeriesMainInformation, u32)>),
}

#[derive(Default)]
pub struct StatisticsTab {
    series_infos_and_time: Vec<(SeriesMainInformation, u32)>,
}

impl StatisticsTab {
    pub fn refresh(&self) -> Command<Message> {
        Command::perform(
            get_series_with_runtime(),
            Message::SeriesInfosAndTimeReceived,
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SeriesInfosAndTimeReceived(series_infos_and_time) => {
                self.series_infos_and_time = series_infos_and_time
            }
        }
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let content = column![
            watch_count(),
            time_count(&self.series_infos_and_time),
            series_list(&self.series_infos_and_time)
        ];
        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Get the collection of all series with their associated total
/// average runtime
async fn get_series_with_runtime() -> Vec<(SeriesMainInformation, u32)> {
    let series_ids_handles: Vec<_> = database::DB
        .get_series_collection()
        .into_iter()
        .map(|series| tokio::spawn(async move { series.get_total_average_runtime().await }))
        .collect();

    let mut infos_and_time = Vec::with_capacity(series_ids_handles.len());
    for handle in series_ids_handles {
        // let info_and_time = handle.await.unwrap();x
        if let Some(info_and_time) = handle
            .await
            .expect("failed to await all series_infos and their average runtime")
        {
            infos_and_time.push(info_and_time);
        }
    }
    infos_and_time
}

impl Tab for StatisticsTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "Statistics".to_owned()
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text("Statistics icon".to_owned())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::Statistics)
    }
}
