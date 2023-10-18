use std::sync::mpsc;

use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{column, container, row, scrollable, text};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Wrap;

use crate::core::{api::tv_maze::series_information::SeriesMainInformation, database};
use crate::gui::assets::icons::GRAPH_UP_ARROW;
use crate::gui::styles;
use series_banner::{
    IndexedMessage as SeriesBannerIndexedMessage, Message as SeriesBannerMessage, SeriesBanner,
};

use mini_widgets::*;

use super::Tab;

mod mini_widgets;

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfosAndTimeReceived(Vec<(SeriesMainInformation, Option<u32>)>),
    SeriesBanner(SeriesBannerIndexedMessage<SeriesBannerMessage>),
    PageScrolled(Viewport),
}

pub struct StatisticsTab {
    series_infos_and_time: Vec<(SeriesMainInformation, Option<u32>)>,
    series_banners: Vec<SeriesBanner>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    scrollable_offset: RelativeOffset,
}

impl StatisticsTab {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
        scrollable_offset: Option<RelativeOffset>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                series_infos_and_time: vec![],
                series_banners: vec![],
                series_page_sender,
                scrollable_offset: scrollable_offset.unwrap_or(RelativeOffset::START),
            },
            Command::perform(
                get_series_with_runtime(),
                Message::SeriesInfosAndTimeReceived,
            ),
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
                    let (banner, banner_command) = SeriesBanner::new(
                        index,
                        series_info_and_time,
                        self.series_page_sender.clone(),
                    );
                    banners.push(banner);
                    banners_commands.push(banner_command);
                }
                self.series_banners = banners;
                Command::batch(banners_commands).map(Message::SeriesBanner)
            }
            Message::SeriesBanner(message) => {
                self.series_banners[message.index()].update(message);
                Command::none()
            }
            Message::PageScrolled(view_port) => {
                self.scrollable_offset = view_port.relative_offset();
                Command::none()
            }
        }
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let series_list: Element<'_, Message, Renderer> = if self.series_banners.is_empty() {
            text("Your watched series will appear here").into()
        } else {
            Wrap::with_elements(
                self.series_banners
                    .iter()
                    .map(|banner| banner.view().map(Message::SeriesBanner))
                    .collect(),
            )
            .spacing(5.0)
            .line_spacing(5.0)
            .into()
        };

        let series_list = container(series_list).width(Length::Fill).center_x();

        let series_infos: Vec<&SeriesMainInformation> = self
            .series_infos_and_time
            .iter()
            .map(|(series_info, _)| series_info)
            .collect();

        let content = column![
            row![
                watch_count(),
                genre_stats(series_infos),
                time_count(&self.series_infos_and_time)
            ]
            .height(200)
            .spacing(10),
            series_list
        ]
        .spacing(10)
        .padding(10);

        container(
            scrollable(content)
                .id(Self::scrollable_id())
                .on_scroll(Message::PageScrolled)
                .direction(styles::scrollable_styles::vertical_direction()),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

/// Get the collection of all series with their associated total
/// average runtime
async fn get_series_with_runtime() -> Vec<(SeriesMainInformation, Option<u32>)> {
    let series_ids_handles: Vec<_> = database::DB
        .get_series_collection()
        .into_iter()
        .map(|series| tokio::spawn(async move { series.get_total_average_watchtime().await }))
        .collect();

    let mut infos_and_time = Vec::with_capacity(series_ids_handles.len());
    for handle in series_ids_handles {
        infos_and_time.push(
            handle
                .await
                .expect("failed to await all series_infos and their average runtime"),
        );
    }
    infos_and_time
}

impl Tab for StatisticsTab {
    type Message = Message;

    fn title() -> &'static str {
        "Statistics"
    }

    fn icon_bytes() -> &'static [u8] {
        GRAPH_UP_ARROW
    }

    fn get_scrollable_offset(&self) -> scrollable::RelativeOffset {
        self.scrollable_offset
    }
}
