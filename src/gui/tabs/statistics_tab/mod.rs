use std::sync::mpsc;

use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{column, container, row, scrollable};
use iced::{Element, Length, Task};
use iced_aw::Wrap;

use crate::core::program_state::ProgramState;
use crate::core::{api::tv_maze::series_information::SeriesMainInformation, database};
use crate::gui::assets::icons::GRAPH_UP_ARROW;
use crate::gui::styles;
use series_banner::{IndexedMessage, Message as SeriesBannerMessage, SeriesBanner};

use mini_widgets::*;

use super::tab_searching::{unavailable_posters, Message as SearcherMessage, Searchable, Searcher};
use super::Tab;

mod mini_widgets;

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfosAndTimeReceived(Vec<(SeriesMainInformation, Option<u32>)>),
    SeriesBanner(IndexedMessage<usize, SeriesBannerMessage>),
    PageScrolled(Viewport),
    Searcher(SearcherMessage),
}

pub struct StatisticsTab<'a> {
    series_infos_and_time: Vec<(SeriesMainInformation, Option<u32>)>,
    series_banners: Vec<SeriesBanner<'a>>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    scrollable_offset: RelativeOffset,
    /// A collection of matched series id after a fuzzy search
    matched_id_collection: Option<Vec<u32>>,
    searcher: Searcher,
    program_state: ProgramState,
}

impl<'a> StatisticsTab<'a> {
    pub fn new(
        program_state: ProgramState,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
        scrollable_offset: Option<RelativeOffset>,
    ) -> (Self, Task<Message>) {
        let db = program_state.get_db();
        (
            Self {
                series_infos_and_time: vec![],
                series_banners: vec![],
                series_page_sender,
                scrollable_offset: scrollable_offset.unwrap_or(RelativeOffset::START),
                matched_id_collection: None,
                searcher: Searcher::new("Search Statistics".to_owned()),
                program_state,
            },
            Task::perform(
                get_series_with_runtime(db),
                Message::SeriesInfosAndTimeReceived,
            ),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SeriesInfosAndTimeReceived(mut series_infos_and_time) => {
                self.series_infos_and_time
                    .clone_from(&series_infos_and_time);

                series_infos_and_time.sort_by(|(_, average_minutes_a), (_, average_minutes_b)| {
                    average_minutes_b.cmp(average_minutes_a)
                });

                let mut banners = Vec::with_capacity(series_infos_and_time.len());
                let mut banners_commands = Vec::with_capacity(series_infos_and_time.len());
                for (index, series_info_and_time) in series_infos_and_time.into_iter().enumerate() {
                    let (banner, banner_command) = SeriesBanner::new(
                        self.program_state.clone(),
                        index,
                        std::borrow::Cow::Owned(series_info_and_time.0),
                        series_info_and_time.1,
                        self.series_page_sender.clone(),
                    );
                    banners.push(banner);
                    banners_commands.push(banner_command);
                }
                self.series_banners = banners;
                Task::batch(banners_commands).map(Message::SeriesBanner)
            }
            Message::SeriesBanner(message) => {
                self.series_banners[message.index()].update(message);
                Task::none()
            }
            Message::PageScrolled(view_port) => {
                self.scrollable_offset = view_port.relative_offset();
                Task::none()
            }
            Message::Searcher(message) => {
                self.searcher.update(message);
                let current_search_term = self.searcher.current_search_term().to_owned();
                self.update_matches(&current_search_term);
                Task::none()
            }
        }
    }
    pub fn view(&self) -> Element<Message> {
        let series_list: Element<'_, Message> = if self.series_banners.is_empty() {
            Self::empty_statistics_posters()
        } else {
            let series_list: Vec<Element<'_, Message>> = self
                .series_banners
                .iter()
                .filter(|banner| {
                    if let Some(matched_id_collection) = &self.matched_id_collection {
                        self.is_matched_id(matched_id_collection, banner.get_series_info().id)
                    } else {
                        true
                    }
                })
                .map(|banner| banner.view().map(Message::SeriesBanner))
                .collect();

            if series_list.is_empty() {
                Self::no_search_matches()
            } else {
                let series_list = Wrap::with_elements(series_list)
                    .spacing(5.0)
                    .line_spacing(5.0);

                scrollable(container(series_list).padding(10).center_x(Length::Fill))
                    .id(Self::scrollable_id())
                    .on_scroll(Message::PageScrolled)
                    .direction(styles::scrollable_styles::vertical_direction())
                    .into()
            }
        };

        let series_infos: Vec<&SeriesMainInformation> = self
            .series_infos_and_time
            .iter()
            .map(|(series_info, _)| series_info)
            .collect();

        let searcher = self.searcher.view().map(Message::Searcher);

        column![
            row![
                watch_count(self.program_state.get_db()),
                genre_stats(series_infos),
                time_count(&self.series_infos_and_time)
            ]
            .height(200)
            .spacing(10),
            searcher,
            series_list
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn empty_statistics_posters() -> Element<'static, Message> {
        unavailable_posters("Your watched series will appear here")
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn no_search_matches() -> Element<'static, Message> {
        unavailable_posters("No matches found!")
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Get the collection of all series with their associated total
/// average runtime
async fn get_series_with_runtime(db: sled::Db) -> Vec<(SeriesMainInformation, Option<u32>)> {
    let series_ids_handles: Vec<_> = database::series_tree::get_series_collection(db)
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

impl<'a> Tab for StatisticsTab<'a> {
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

impl<'a> Searchable for StatisticsTab<'a> {
    fn get_series_information_collection(&self) -> Vec<&SeriesMainInformation> {
        self.series_banners
            .iter()
            .map(|banner| banner.get_series_info())
            .collect()
    }

    fn matches_id_collection(&mut self) -> &mut Option<Vec<u32>> {
        &mut self.matched_id_collection
    }
}
