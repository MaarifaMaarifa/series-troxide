use iced::widget::{container, scrollable, Column};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::caching::episode_list::EpisodeList;
use crate::gui::assets::icons::CARD_CHECKLIST;
use crate::gui::troxide_widget;
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::{
    core::{caching, database},
    gui::{Message as GuiMessage, Tab},
};

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

#[derive(Default)]
pub struct WatchlistTab {
    series_posters: Vec<(SeriesPoster, usize)>,
    load_state: LoadState,
}

impl WatchlistTab {
    pub fn refresh(&self) -> Command<Message> {
        Command::perform(
            get_series_informations_and_watched_episodes(),
            Message::SeriesInformationLoaded,
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
            Message::SeriesPoster(index, message) => self.series_posters[index]
                .0
                .update(*message)
                .map(|message| {
                    Message::SeriesPoster(message.get_id().unwrap_or(0), Box::new(message))
                }),
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
                let watchlist_items: Vec<Element<'_, Message, Renderer>> = self
                    .series_posters
                    .iter()
                    .map(|(poster, total_episodes)| {
                        poster.watchlist_view(*total_episodes).map(|message| {
                            Message::SeriesPoster(message.get_id().unwrap_or(0), Box::new(message))
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

/// checks of the given series has pending episodes to be watched in the database. That given series
/// is provides through it's EpisodeList Structure.
fn has_pending_episodes(database_series: &database::Series, episodes_list: &EpisodeList) -> bool {
    episodes_list.get_total_watchable_episodes() != database_series.get_total_episodes()
}

/// Get the series ids of all the series that have pending episodes to be watched
async fn get_pending_series_ids() -> Vec<u32> {
    let ids = database::DB.get_ids_and_series();
    let episode_lists_handles: Vec<_> = ids
        .iter()
        .map(|(id, _)| {
            let id = id.parse::<u32>().expect("could not parse series id");
            tokio::spawn(caching::episode_list::EpisodeList::new(id))
        })
        .collect();

    let mut episodes_lists = Vec::with_capacity(episode_lists_handles.len());
    for handle in episode_lists_handles {
        let episode_list = handle
            .await
            .expect("failed to await episode list handle")
            .expect("failed to get episode list");

        episodes_lists.push(episode_list);
    }

    ids.iter()
        .zip(episodes_lists.iter())
        .filter(|((_, series), _)| series.is_tracked())
        .filter(|((_, series), episode_list)| has_pending_episodes(series, episode_list))
        .map(|((id, _), _)| id.parse::<u32>().expect("could not parse series id"))
        .collect()
}

async fn get_series_informations_and_watched_episodes() -> Vec<(SeriesMainInformation, usize)> {
    let handles: Vec<_> = get_pending_series_ids()
        .await
        .into_iter()
        .map(|id| {
            tokio::spawn(async move {
                let series_information_res =
                    caching::series_information::get_series_main_info_with_id(id)
                        .await
                        .expect("failed to get series information");
                let total_episode = caching::episode_list::EpisodeList::new(id)
                    .await
                    .expect("failed to get series episode list")
                    .get_total_watchable_episodes();
                (series_information_res, total_episode)
            })
        })
        .collect();

    let mut series_informations = Vec::with_capacity(handles.len());
    for handle in handles {
        let series_info = handle
            .await
            .expect("failed to await when requesting series information");
        series_informations.push(series_info);
    }
    series_informations
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
