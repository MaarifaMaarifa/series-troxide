use iced::widget::{column, container, scrollable, Column};
use iced::{theme, Command, Element, Length, Renderer};

use crate::gui::styles;
use crate::{
    core::{api::series_information::SeriesMainInformation, database},
    gui::{Message as GuiMessage, Tab},
};
use series_banner::{Message as SeriesBannerMessage, SeriesBanner};

use mini_widgets::*;

mod mini_widgets;

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfosAndTimeReceived(Vec<(SeriesMainInformation, u32)>),
    SeriesBanner(usize, SeriesBannerMessage),
}

#[derive(Default)]
pub struct StatisticsTab {
    series_infos_and_time: Vec<(SeriesMainInformation, u32)>,
    series_banners: Vec<SeriesBanner>,
}

impl StatisticsTab {
    pub fn refresh(&self) -> Command<Message> {
        Command::perform(
            get_series_with_runtime(),
            Message::SeriesInfosAndTimeReceived,
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesInfosAndTimeReceived(mut series_infos_and_time) => {
                self.series_infos_and_time = series_infos_and_time.clone();

                series_infos_and_time.sort_by(|(_, average_minutes_a), (_, average_minutes_b)| {
                    average_minutes_b.cmp(average_minutes_a)
                });

                let mut banners = Vec::with_capacity(series_infos_and_time.len());
                let mut banners_commands = Vec::with_capacity(series_infos_and_time.len());
                for (index, series_info_and_time) in series_infos_and_time.into_iter().enumerate() {
                    let (banner, banner_command) = SeriesBanner::new(index, series_info_and_time);
                    banners.push(banner);
                    banners_commands.push(banner_command);
                }
                self.series_banners = banners;
                Command::batch(banners_commands)
                    .map(|message| Message::SeriesBanner(message.get_id(), message))
            }
            Message::SeriesBanner(index, message) => {
                self.series_banners[index].update(message);
                Command::none()
            }
        }
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let series_list = Column::with_children(
            self.series_banners
                .iter()
                .map(|banner| {
                    banner
                        .view()
                        .map(|message| Message::SeriesBanner(message.get_id(), message))
                })
                .collect(),
        );

        let series_list =
            container(series_list).style(styles::container_styles::first_class_container_theme());

        let content = column![
            watch_count(),
            time_count(&self.series_infos_and_time),
            series_list
        ]
        .spacing(10)
        .padding(10);

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
        iced_aw::TabLabel::Text(self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::Statistics)
    }
}
