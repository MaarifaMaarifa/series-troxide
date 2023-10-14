use std::sync::mpsc;

use iced::widget::{container, scrollable, text, Column};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

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

use super::Tab;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationLoaded(Vec<(SeriesMainInformation, Option<Episode>, usize)>),
    SeriesPoster(SeriesPosterIndexedMessage<SeriesPosterMessage>),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct WatchlistTab {
    series_posters: Vec<(SeriesPoster, Option<Episode>, usize)>,
    load_state: LoadState,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
}

impl WatchlistTab {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                series_posters: vec![],
                load_state: LoadState::Loading,
                series_page_sender,
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
                self.series_posters = posters;
                Command::batch(commands).map(Message::SeriesPoster)
            }
            Message::SeriesPoster(message) => self.series_posters[message.index()]
                .0
                .update(message)
                .map(Message::SeriesPoster),
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

                    scrollable(
                        Column::with_children(watchlist_items)
                            .padding(5)
                            .spacing(5)
                            .align_items(iced::Alignment::Center)
                            .width(Length::Fill),
                    )
                    .direction(styles::scrollable_styles::vertical_direction())
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
    fn title() -> &'static str {
        "Watchlist"
    }

    fn icon_bytes() -> &'static [u8] {
        CARD_CHECKLIST
    }
}
