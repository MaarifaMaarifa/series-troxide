use std::sync::mpsc;

use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{column, container, scrollable, text, Column, Space};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

use super::Tab;
use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::caching::episode_list::EpisodeList;
use crate::core::caching::series_list;
use crate::core::{caching, database};
use crate::gui::assets::icons::CARD_CHECKLIST;
use crate::gui::styles;
use crate::gui::troxide_widget::series_poster::{
    IndexedMessage as SeriesPosterIndexedMessage, Message as SeriesPosterMessage, SeriesPoster,
};
use watchlist_summary::WatchlistSummary;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationLoaded(Vec<(SeriesMainInformation, Option<Episode>, usize)>),
    SeriesPoster(SeriesPosterIndexedMessage<SeriesPosterMessage>),
    PageScrolled(Viewport),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct WatchlistTab {
    series_posters: Vec<(SeriesPoster, Option<Episode>, usize)>,
    watchlist_summary: Option<WatchlistSummary>,
    load_state: LoadState,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    scrollable_offset: RelativeOffset,
}

impl WatchlistTab {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
        scrollable_offset: Option<RelativeOffset>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                series_posters: vec![],
                watchlist_summary: None,
                load_state: LoadState::Loading,
                series_page_sender,
                scrollable_offset: scrollable_offset.unwrap_or(RelativeOffset::START),
            },
            Command::perform(
                get_series_informations_and_watched_episodes(),
                Message::SeriesInformationLoaded,
            ),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesInformationLoaded(mut series_infos) => {
                self.load_state = LoadState::Loaded;

                // Arranging the watchlist shows alphabetically
                series_infos.sort_by_key(|(series_info, _, _)| series_info.name.clone());

                let mut posters = Vec::with_capacity(series_infos.len());
                let mut commands = Vec::with_capacity(series_infos.len());
                for (index, (info, episode, total_episodes)) in series_infos.into_iter().enumerate()
                {
                    let (poster, command) =
                        SeriesPoster::new(index, info, self.series_page_sender.clone());
                    posters.push((poster, episode, total_episodes));
                    commands.push(command);
                }

                self.watchlist_summary = Some(WatchlistSummary::new(
                    posters
                        .iter()
                        .map(|(_, _, total_episodes)| total_episodes)
                        .sum(),
                    posters
                        .iter()
                        .map(|(poster, _, _)| {
                            let watched_episodes = database::DB
                                .get_series(poster.get_series_info().id)
                                .map(|series| series.get_total_episodes())
                                .unwrap_or(0);

                            watched_episodes
                        })
                        .sum(),
                    posters
                        .iter()
                        .map(|(poster, _, total_episodes)| {
                            let series_info = poster.get_series_info();

                            let watched_episodes = database::DB
                                .get_series(series_info.id)
                                .map(|series| series.get_total_episodes())
                                .unwrap_or(0);

                            (total_episodes - watched_episodes) as u32
                                * series_info.average_runtime.unwrap_or_default()
                        })
                        .sum(),
                    posters.len(),
                ));
                self.series_posters = posters;

                Command::batch(commands).map(Message::SeriesPoster)
            }
            Message::SeriesPoster(message) => self.series_posters[message.index()]
                .0
                .update(message)
                .map(Message::SeriesPoster),
            Message::PageScrolled(view_port) => {
                self.scrollable_offset = view_port.relative_offset();
                Command::none()
            }
        }
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        match self.load_state {
            LoadState::Loading => container(Spinner::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            LoadState::Loaded => {
                if self.series_posters.is_empty() {
                    container(
                        text("All Clear!")
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                    )
                    .center_x()
                    .center_y()
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
                } else {
                    let watchlist_items: Vec<Element<'_, Message, Renderer>> = self
                        .series_posters
                        .iter()
                        .map(|(poster, last_watched_episode, total_episodes)| {
                            poster
                                .watchlist_view(last_watched_episode.as_ref(), *total_episodes)
                                .map(Message::SeriesPoster)
                        })
                        .collect();

                    let watchlist_summary = self
                        .watchlist_summary
                        .as_ref()
                        .map(|watchlist_summary| watchlist_summary.view())
                        .unwrap_or(Space::new(0, 0).into());

                    let watchlist_items = Column::with_children(watchlist_items)
                        .spacing(5)
                        .align_items(iced::Alignment::Center)
                        .width(Length::Fill);

                    let content = column![watchlist_summary, watchlist_items]
                        .padding(5)
                        .spacing(10)
                        .align_items(iced::Alignment::Center);

                    scrollable(content)
                        .direction(styles::scrollable_styles::vertical_direction())
                        .id(Self::scrollable_id())
                        .on_scroll(Message::PageScrolled)
                        .into()
                }
            }
        }
    }
}

/// checks of the given series has pending episodes to be watched in the database. That given series
/// is provided through it's EpisodeList Structure.
fn has_pending_episodes(database_series: &database::Series, episodes_list: &EpisodeList) -> bool {
    episodes_list.get_total_watchable_episodes() != database_series.get_total_episodes()
}

async fn get_series_informations_and_watched_episodes(
) -> Vec<(SeriesMainInformation, Option<Episode>, usize)> {
    let tracked_series_informations = series_list::SeriesList::new()
        .get_tracked_series_informations()
        .await
        .unwrap();

    let episode_lists_handles: Vec<_> = tracked_series_informations
        .iter()
        .map(|series_info| tokio::spawn(caching::episode_list::EpisodeList::new(series_info.id)))
        .collect();

    let mut episodes_lists = Vec::with_capacity(episode_lists_handles.len());
    for handle in episode_lists_handles {
        let episode_list = handle
            .await
            .expect("failed to await episode list handle")
            .expect("failed to get episode list");

        episodes_lists.push(episode_list);
    }

    tracked_series_informations
        .into_iter()
        .zip(episodes_lists.into_iter())
        .filter(|(series_info, episode_list)| {
            let series = database::DB.get_series(series_info.id).unwrap();
            has_pending_episodes(&series, episode_list)
        })
        .map(|(series_info, episode_list)| {
            let series = database::DB.get_series(series_info.id).unwrap();

            // Finding an episode that is not watched, making it as the next episode to watch
            let next_episode_to_watch = episode_list.get_all_episodes().iter().find(|episode| {
                series
                    .get_season(episode.season)
                    .map(|season| {
                        episode
                            .number
                            .map(|episode_number| !season.is_episode_watched(episode_number))
                            .unwrap_or(false)
                    })
                    .unwrap_or(true) // if season isn't watched, let's get it's first episode
            });
            (
                series_info,
                next_episode_to_watch.cloned(),
                episode_list.get_total_watchable_episodes(),
            )
        })
        .collect()
}

impl Tab for WatchlistTab {
    type Message = Message;

    fn title() -> &'static str {
        "Watchlist"
    }

    fn icon_bytes() -> &'static [u8] {
        CARD_CHECKLIST
    }

    fn get_scrollable_offset(&self) -> scrollable::RelativeOffset {
        self.scrollable_offset
    }
}

mod watchlist_summary {
    use crate::gui::helpers::time::SaneTime;
    use crate::gui::styles;

    use super::Message;
    use iced::widget::{column, container, progress_bar, row, text};
    use iced::{Alignment, Element, Renderer};

    pub struct WatchlistSummary {
        total_episodes_watched: usize,
        total_episodes: usize,
        total_minutes: u32,
        total_shows_to_watch: usize,
    }

    impl WatchlistSummary {
        pub fn new(
            total_episodes: usize,
            total_episodes_watched: usize,
            total_minutes: u32,
            total_shows_to_watch: usize,
        ) -> Self {
            Self {
                total_episodes,
                total_episodes_watched,
                total_minutes,
                total_shows_to_watch,
            }
        }

        pub fn view(&self) -> Element<'static, Message, Renderer> {
            let total_shows_to_watch = Self::summary_item(
                "Total Series to Watch",
                self.total_shows_to_watch.to_string(),
            );

            let total_time_to_watch = Self::summary_item(
                "Total Time Required to Watch",
                SaneTime::new(self.total_minutes).to_string(),
            );

            let episodes_left_to_watch = self.total_episodes - self.total_episodes_watched;

            let total_episodes = Self::summary_item(
                "Total Episodes to Watch",
                (episodes_left_to_watch).to_string(),
            );

            let percentage_progress = Self::summary_item(
                "Progress",
                format!(
                    "{}%",
                    ((self.total_episodes_watched as f32 / self.total_episodes as f32) * 100_f32)
                        .trunc()
                ),
            );

            let progress = row![
                progress_bar(
                    0.0..=self.total_episodes as f32,
                    self.total_episodes_watched as f32,
                )
                .height(10)
                .width(500),
                text(format!(
                    "{}/{}",
                    self.total_episodes_watched as f32, self.total_episodes as f32
                ))
            ]
            .spacing(5);

            let content = column![
                row![
                    percentage_progress,
                    total_time_to_watch,
                    total_shows_to_watch,
                    total_episodes,
                ]
                .spacing(20),
                progress,
            ]
            .spacing(20)
            .align_items(Alignment::Center);

            container(content)
                .padding(20)
                .style(styles::container_styles::first_class_container_rounded_theme())
                .into()
        }

        fn summary_item(title: &'static str, info: String) -> Element<'static, Message, Renderer> {
            column![
                text(title),
                text(info)
                    .style(styles::text_styles::accent_color_theme())
                    .size(18)
            ]
            .align_items(Alignment::Center)
            .into()
        }
    }
}
