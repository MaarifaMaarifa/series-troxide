use std::sync::mpsc;

use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{column, container, scrollable, Column, Space};
use iced::{Command, Element, Length};
use iced_aw::Spinner;

use super::tab_searching::{unavailable_posters, Message as SearcherMessage, Searchable, Searcher};
use super::Tab;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::caching::episode_list::EpisodeList;
use crate::core::caching::series_list;
use crate::core::{caching, database};
use crate::gui::assets::icons::CARD_CHECKLIST;
use crate::gui::message::IndexedMessage;
use crate::gui::styles;
use watchlist_poster::{Message as WatchlistPosterMessage, WatchlistPoster};
use watchlist_summary::WatchlistSummary;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationLoaded(Vec<(SeriesMainInformation, EpisodeList, usize)>),
    WatchlistPoster(IndexedMessage<usize, WatchlistPosterMessage>),
    PageScrolled(Viewport),
    Searcher(SearcherMessage),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct WatchlistTab<'a> {
    load_state: LoadState,
    watchlist_posters: Vec<WatchlistPoster<'a>>,
    watchlist_summary: Option<WatchlistSummary>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    scrollable_offset: RelativeOffset,
    /// A collection of matched series id after a fuzzy search
    matched_id_collection: Option<Vec<u32>>,
    searcher: Searcher,
}

impl<'a> WatchlistTab<'a> {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
        scrollable_offset: Option<RelativeOffset>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                watchlist_posters: vec![],
                watchlist_summary: None,
                load_state: LoadState::Loading,
                series_page_sender,
                scrollable_offset: scrollable_offset.unwrap_or(RelativeOffset::START),
                matched_id_collection: None,
                searcher: Searcher::new("Search Watchlist".to_owned()),
            },
            Command::perform(
                get_series_information_and_watched_episodes(),
                Message::SeriesInformationLoaded,
            ),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesInformationLoaded(mut series_infos) => {
                self.load_state = LoadState::Loaded;

                self.watchlist_summary = Some(WatchlistSummary::new(
                    series_infos
                        .iter()
                        .map(|(_, _, total_episodes)| total_episodes)
                        .sum(),
                    series_infos
                        .iter()
                        .map(|(series_info, _, total_episodes)| {
                            (
                                series_info.id,
                                *total_episodes as u32,
                                series_info.average_runtime.unwrap_or_default(),
                            )
                        })
                        .collect(),
                ));

                // Arranging the watchlist shows alphabetically
                series_infos.sort_by_key(|(series_info, _, _)| series_info.name.clone());

                let mut posters = Vec::with_capacity(series_infos.len());
                let mut commands = Vec::with_capacity(series_infos.len());
                for (index, (info, episode, total_episodes)) in series_infos.into_iter().enumerate()
                {
                    let (poster, command) = WatchlistPoster::new(
                        index,
                        std::borrow::Cow::Owned(info),
                        episode,
                        total_episodes,
                        self.series_page_sender.clone(),
                    );
                    posters.push(poster);
                    commands.push(command);
                }

                self.watchlist_posters = posters;

                Command::batch(commands).map(Message::WatchlistPoster)
            }
            Message::WatchlistPoster(message) => self.watchlist_posters[message.index()]
                .update(message)
                .map(Message::WatchlistPoster),
            Message::PageScrolled(view_port) => {
                self.scrollable_offset = view_port.relative_offset();
                Command::none()
            }
            Message::Searcher(message) => {
                self.searcher.update(message);
                let current_search_term = self.searcher.current_search_term().to_owned();
                self.update_matches(&current_search_term);
                Command::none()
            }
        }
    }
    pub fn view(&self) -> Element<Message> {
        match self.load_state {
            LoadState::Loading => container(Spinner::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            LoadState::Loaded => {
                if self.watchlist_posters.is_empty() {
                    Self::empty_watchlist_posters()
                } else {
                    let watchlist_summary = self
                        .watchlist_summary
                        .as_ref()
                        .map(|watchlist_summary| watchlist_summary.view())
                        .unwrap_or(Space::new(0, 0).into());

                    let watchlist_items: Vec<Element<'_, Message>> = self
                        .watchlist_posters
                        .iter()
                        .filter(|poster| {
                            if let Some(matched_id_collection) = &self.matched_id_collection {
                                self.is_matched_id(
                                    matched_id_collection.as_slice(),
                                    poster.get_series_info().id,
                                )
                            } else {
                                true
                            }
                        })
                        .map(|poster| poster.view().map(Message::WatchlistPoster))
                        .collect();

                    let watchlist_items = if watchlist_items.is_empty() {
                        Self::no_search_matches()
                    } else {
                        let watchlist_items = Column::with_children(watchlist_items)
                            .spacing(5)
                            .align_items(iced::Alignment::Center)
                            .width(Length::Fill);

                        // We are wrapping the watchlist_items into a container so that the scrollbar does not touch the watchlist
                        // element by setting up padding in the container
                        scrollable(container(watchlist_items).padding(10).width(Length::Fill))
                            .direction(styles::scrollable_styles::vertical_direction())
                            .id(Self::scrollable_id())
                            .on_scroll(Message::PageScrolled)
                            .into()
                    };

                    let searcher = self.searcher.view().map(Message::Searcher);

                    column![watchlist_summary, searcher, watchlist_items]
                        .padding(5)
                        .spacing(10)
                        .align_items(iced::Alignment::Center)
                        .into()
                }
            }
        }
    }

    fn empty_watchlist_posters() -> Element<'static, Message> {
        unavailable_posters("All Clear!")
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

/// checks of the given series has pending episodes to be watched in the database. That given series
/// is provided through it's EpisodeList Structure.
fn has_pending_episodes(database_series: &database::Series, episodes_list: &EpisodeList) -> bool {
    episodes_list.get_total_watchable_episodes() != database_series.get_total_episodes()
}

async fn get_series_information_and_watched_episodes(
) -> Vec<(SeriesMainInformation, EpisodeList, usize)> {
    let tracked_series_information = series_list::SeriesList::new()
        .get_tracked_series_information()
        .await
        .unwrap();

    let episode_lists_handles: Vec<_> = tracked_series_information
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

    tracked_series_information
        .into_iter()
        .zip(episodes_lists.into_iter())
        .filter(|(series_info, episode_list)| {
            let series = database::DB.get_series(series_info.id).unwrap();
            has_pending_episodes(&series, episode_list)
        })
        .map(|(series_info, episode_list)| {
            let total_watchable_episodes = episode_list.get_total_watchable_episodes();
            (series_info, episode_list, total_watchable_episodes)
        })
        .collect()
}

impl<'a> Tab for WatchlistTab<'a> {
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

impl<'a> Searchable for WatchlistTab<'a> {
    fn get_series_information_collection(&self) -> Vec<&SeriesMainInformation> {
        self.watchlist_posters
            .iter()
            .map(|poster| poster.get_series_info())
            .collect()
    }

    fn matches_id_collection(&mut self) -> &mut Option<Vec<u32>> {
        &mut self.matched_id_collection
    }
}

mod watchlist_poster {
    use std::sync::mpsc;

    use iced::widget::{
        button, column, container, horizontal_rule, image, mouse_area, progress_bar, row, text,
        Space,
    };
    use iced::{Command, Element, Length};

    use crate::core::api::tv_maze::series_information::SeriesMainInformation;
    use crate::core::caching::episode_list::EpisodeList;
    use crate::core::database;
    use crate::gui::helpers::{self, season_episode_str_gen};
    use crate::gui::styles;
    use crate::gui::troxide_widget::episode_widget::{
        Episode as EpisodePoster, Message as EpisodePosterMessage, PosterType,
    };
    use crate::gui::troxide_widget::series_poster::{
        GenericPoster, GenericPosterMessage, IndexedMessage,
    };

    #[derive(Debug, Clone)]
    pub enum Message {
        Poster(GenericPosterMessage),
        EpisodePoster(IndexedMessage<usize, EpisodePosterMessage>),
        SeriesPosterPressed,
        ToggleEpisodeInfo,
    }

    pub struct WatchlistPoster<'a> {
        index: usize,
        poster: GenericPoster<'a>,
        episode_list: EpisodeList,
        total_series_episodes: usize,
        episode_poster: Option<EpisodePoster>,
        current_poster_id: usize,
        show_episode_info: bool,
    }

    impl<'a> WatchlistPoster<'a> {
        pub fn new(
            index: usize,
            series_info: std::borrow::Cow<'a, SeriesMainInformation>,
            episode_list: EpisodeList,
            total_series_episodes: usize,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (Self, Command<IndexedMessage<usize, Message>>) {
            let (poster, poster_command) = GenericPoster::new(series_info, series_page_sender);

            (
                Self {
                    index,
                    poster,
                    episode_list,
                    total_series_episodes,
                    episode_poster: None,
                    current_poster_id: 0,
                    show_episode_info: false,
                },
                poster_command
                    .map(Message::Poster)
                    .map(move |message| IndexedMessage::new(index, message)),
            )
        }

        pub fn get_series_info(&self) -> &SeriesMainInformation {
            self.poster.get_series_info()
        }

        pub fn update(
            &mut self,
            message: IndexedMessage<usize, Message>,
        ) -> Command<IndexedMessage<usize, Message>> {
            let command = match message.message() {
                Message::Poster(message) => {
                    self.poster.update(message);
                    Command::none()
                }
                Message::SeriesPosterPressed => {
                    self.poster.open_series_page();
                    Command::none()
                }
                Message::ToggleEpisodeInfo => {
                    self.show_episode_info = !self.show_episode_info;

                    if self.episode_poster.is_none() {
                        self.update_episode_poster()
                    } else {
                        Command::none()
                    }
                }
                Message::EpisodePoster(message) => {
                    if message.index() != self.current_poster_id {
                        Command::none()
                    } else if let Some(episode_poster) = self.episode_poster.as_mut() {
                        let index = self.index;
                        episode_poster.update(message).map(move |message| {
                            IndexedMessage::new(index, Message::EpisodePoster(message))
                        })
                    } else {
                        // This situation can happen when all the episodes have been marked watched
                        Command::none()
                    }
                }
            };

            let episode_update_command = if self
                .episode_poster
                .as_ref()
                .map(|poster| poster.is_set_watched())
                .unwrap_or(false)
            {
                self.episode_poster = None;
                self.update_episode_poster()
            } else {
                Command::none()
            };

            Command::batch([episode_update_command, command])
        }

        fn update_episode_poster(&mut self) -> Command<IndexedMessage<usize, Message>> {
            self.current_poster_id += 1;

            if let Some(episode) = self.episode_list.get_next_episode_to_watch() {
                let (episode_poster, episode_poster_command) = EpisodePoster::new(
                    self.current_poster_id,
                    self.poster.get_series_info().id,
                    self.poster.get_series_info().name.clone(),
                    episode.clone(),
                );
                let index = self.index;
                self.episode_poster = Some(episode_poster);
                episode_poster_command
                    .map(move |message| IndexedMessage::new(index, Message::EpisodePoster(message)))
            } else {
                Command::none()
            }
        }

        pub fn view(&self) -> Element<'_, IndexedMessage<usize, Message>> {
            let mut content = row!().padding(2).spacing(5);
            if let Some(image_bytes) = self.poster.get_image() {
                let image_handle = image::Handle::from_memory(image_bytes.clone());
                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(Space::new(100, 140));
            };

            let mut metadata = column!().padding(2).spacing(5);

            metadata = metadata.push(
                text(&self.poster.get_series_info().name)
                    .size(18)
                    .style(styles::text_styles::accent_color_theme()),
            );

            let watched_episodes = database::DB
                .get_series(self.poster.get_series_info().id)
                .map(|series| series.get_total_episodes())
                .unwrap_or(0);

            let progress_bar = row![
                progress_bar(
                    0.0..=self.total_series_episodes as f32,
                    watched_episodes as f32,
                )
                .height(10)
                .width(500),
                text(format!(
                    "{}/{}",
                    watched_episodes as f32, self.total_series_episodes as f32
                ))
            ]
            .spacing(5);

            metadata = metadata.push(progress_bar);

            if let Some(next_episode_to_watch) = self.episode_list.get_next_episode_to_watch() {
                let season_number = next_episode_to_watch.season;
                let episode_number = next_episode_to_watch
                    .number
                    .expect("episode should have a valid number at this point");
                let episode_name = next_episode_to_watch.name.as_str();
                let episode_text = text(format!(
                    "{}: {}",
                    season_episode_str_gen(season_number, episode_number),
                    episode_name
                ));
                metadata = metadata.push(episode_text);
            };

            let episodes_left = self.total_series_episodes - watched_episodes;

            metadata = metadata.push(text(format!("{} episodes left", episodes_left)));

            if let Some(runtime) = self.poster.get_series_info().average_runtime {
                metadata = metadata.push(text(helpers::time::NaiveTime::new(
                    runtime * episodes_left as u32,
                )));
            };

            metadata = metadata.push(self.show_episode_info_button());

            content = content.push(metadata);

            let mut content = column![content].spacing(5).width(Length::Fill);

            if let Some(episode_poster) = self.episode_poster.as_ref() {
                if self.show_episode_info {
                    content = content.push(horizontal_rule(1));
                    let episode_view = episode_poster
                        .view(PosterType::Watchlist)
                        .map(Message::EpisodePoster);

                    content = content.push(container(episode_view).width(Length::Fill).center_x());
                }
            }

            let content = container(content)
                .padding(5)
                .style(styles::container_styles::first_class_container_rounded_theme())
                .width(1000);

            let element: Element<'_, Message> = mouse_area(content)
                .on_press(Message::SeriesPosterPressed)
                .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }

        fn show_episode_info_button(&self) -> Element<'static, Message> {
            let content = match self.show_episode_info {
                true => "Hide Episode Info",
                false => "Show Episode Info",
            };

            button(content)
                .on_press(Message::ToggleEpisodeInfo)
                .style(styles::button_styles::transparent_button_with_rounded_border_theme())
                .into()
        }
    }
}

mod watchlist_summary {
    use crate::core::database;
    use crate::gui::helpers::time::NaiveTime;
    use crate::gui::styles;

    use super::Message;
    use iced::widget::{column, container, progress_bar, row, text};
    use iced::{Alignment, Element};

    pub struct WatchlistSummary {
        /// Vec<(series id, total_episodes, series runtime)>
        series_ids: Vec<(u32, u32, u32)>,
        total_episodes: usize,
    }

    impl WatchlistSummary {
        pub fn new(total_episodes: usize, series_ids: Vec<(u32, u32, u32)>) -> Self {
            Self {
                total_episodes,
                series_ids,
            }
        }

        pub fn view(&self) -> Element<'static, Message> {
            let total_shows_to_watch =
                Self::summary_item("Total Series to Watch", self.series_ids.len().to_string());

            let total_time_to_watch = Self::summary_item(
                "Total Time Required to Watch",
                NaiveTime::new(
                    self.series_ids
                        .iter()
                        .map(|(id, total_episodes, time)| {
                            let watched_episodes = database::DB
                                .get_series(*id)
                                .map(|series| series.get_total_episodes())
                                .unwrap_or(0);

                            (total_episodes - watched_episodes as u32) * time
                        })
                        .sum(),
                )
                .to_string(),
            );

            let total_episodes_watched: usize = self
                .series_ids
                .iter()
                .map(|tup| {
                    database::DB
                        .get_series(tup.0)
                        .map(|series| series.get_total_episodes())
                        .unwrap_or(0)
                })
                .sum();

            let episodes_left_to_watch = self.total_episodes - total_episodes_watched;

            let total_episodes = Self::summary_item(
                "Total Episodes to Watch",
                (episodes_left_to_watch).to_string(),
            );

            let percentage_progress = Self::summary_item(
                "Progress",
                format!(
                    "{}%",
                    ((total_episodes_watched as f32 / self.total_episodes as f32) * 100_f32)
                        .trunc()
                ),
            );

            let progress = row![
                progress_bar(
                    0.0..=self.total_episodes as f32,
                    total_episodes_watched as f32,
                )
                .height(10)
                .width(500),
                text(format!(
                    "{}/{}",
                    total_episodes_watched as f32, self.total_episodes as f32
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

        fn summary_item(title: &'static str, info: String) -> Element<'static, Message> {
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
