use std::sync::mpsc;

use iced::widget::{container, scrollable, text, Column};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::caching::episode_list::EpisodeList;
use crate::core::caching::series_list;
use crate::gui::assets::icons::CARD_CHECKLIST;
use crate::gui::troxide_widget;
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::{
    core::{caching, database},
    gui::{Message as GuiMessage, Tab},
};

use super::series_view;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationLoaded(Vec<(SeriesMainInformation, usize)>),
    SeriesPoster(usize, Box<SeriesPosterMessage>),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct WatchlistTab {
    series_posters: Vec<(SeriesPoster, usize)>,
    load_state: LoadState,
    series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
}

impl WatchlistTab {
    pub fn new(
        series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
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
            Message::SeriesInformationLoaded(series_infos) => {
                self.load_state = LoadState::Loaded;
                let mut posters = Vec::with_capacity(series_infos.len());
                let mut commands = Vec::with_capacity(series_infos.len());
                for (index, (info, total_episodes)) in series_infos.into_iter().enumerate() {
                    let (poster, command) = SeriesPoster::new(index, info);
                    posters.push((poster, total_episodes));
                    commands.push(command);
                }
                self.series_posters = posters;
                Command::batch(commands).map(|message| {
                    Message::SeriesPoster(message.get_id().unwrap_or(0), Box::new(message))
                })
            }
            Message::SeriesPoster(index, message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_info) = *message.clone() {
                    self.series_page_sender
                        .send(series_view::Series::from_series_information(*series_info))
                        .expect("failed to send the series page");
                    return Command::none();
                }
                self.series_posters[index]
                    .0
                    .update(*message)
                    .map(|message| {
                        Message::SeriesPoster(message.get_id().unwrap_or(0), Box::new(message))
                    })
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
                        .map(|(poster, total_episodes)| {
                            poster.watchlist_view(*total_episodes).map(|message| {
                                Message::SeriesPoster(
                                    message.get_id().unwrap_or(0),
                                    Box::new(message),
                                )
                            })
                        })
                        .collect();

                    scrollable(
                        Column::with_children(watchlist_items)
                            .padding(5)
                            .spacing(5)
                            .align_items(iced::Alignment::Center)
                            .width(Length::Fill),
                    )
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

async fn get_series_informations_and_watched_episodes() -> Vec<(SeriesMainInformation, usize)> {
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
            (series_info, episode_list.get_total_watchable_episodes())
        })
        .collect()
}

impl Tab for WatchlistTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "Watchlist".to_owned()
    }

    fn tab_label(&self) -> troxide_widget::tabs::TabLabel {
        troxide_widget::tabs::TabLabel::new(self.title(), CARD_CHECKLIST)
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::Watchlist)
    }
}
