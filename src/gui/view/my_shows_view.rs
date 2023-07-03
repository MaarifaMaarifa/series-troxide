use std::collections::HashSet;

use crate::core::api::episodes_information::Episode;
use crate::core::caching;
use crate::core::{api::series_information::SeriesMainInformation, database};
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::gui::troxide_widget::{GREEN_THEME, RED_THEME};
use crate::gui::{Message as GuiMessage, Tab};
use iced::widget::{container, scrollable, Column};
use iced_aw::{Spinner, Wrap};

use iced::Length;
use iced::{
    widget::{column, text},
    Command, Element, Renderer,
};

use super::series_view::SeriesStatus;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationsReceived(Vec<SeriesMainInformation>),
    SeriesSelected(Box<SeriesMainInformation>),
    SeriesPosterAction(usize, SeriesPosterMessage),
    UpcomingReleaseSeriesReceived(Vec<(SeriesMainInformation, (Episode, String))>),
    UpcomingReleasePosterAction(usize, SeriesPosterMessage),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Default)]
pub struct MyShowsTab {
    load_state: LoadState,
    series_ids: Vec<String>,
    upcoming_releases: Vec<(SeriesPoster, (Episode, String))>,
    series: Vec<SeriesPoster>,
}

impl MyShowsTab {
    pub fn refresh(&mut self) -> Command<Message> {
        let series_ids = database::DB.get_series_id_collection();
        let fresh_series_ids: HashSet<String> = series_ids.iter().cloned().collect();

        let current_series_ids: HashSet<String> = self.series_ids.iter().cloned().collect();

        // Preventing my_shows page from reloading when no series updates has occured in the database
        if (self.series.is_empty() && !series_ids.is_empty())
            || (fresh_series_ids != current_series_ids)
        {
            self.load_state = LoadState::Loading;
            self.series_ids = series_ids.clone();

            let series_information =
                caching::series_information::get_series_main_info_with_ids(series_ids);

            return Command::perform(series_information, |series_infos| {
                Message::SeriesInformationsReceived(series_infos)
            });
        } else {
            Command::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesSelected(_) => {
                unimplemented!("My shows page should not handle selecting a series poster")
            }
            Message::SeriesPosterAction(index, message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_info) = message {
                    return Command::perform(async {}, |_| Message::SeriesSelected(series_info));
                }
                return self.series[index]
                    .update(message)
                    .map(move |message| Message::SeriesPosterAction(index, message));
            }
            Message::SeriesInformationsReceived(series_infos) => {
                self.load_state = LoadState::Loaded;

                let upcoming_series_release_command = Command::perform(
                    get_series_release_time(series_infos.clone()),
                    Message::UpcomingReleaseSeriesReceived,
                );

                let mut series_posters = Vec::with_capacity(series_infos.len());
                let mut series_posters_commands = Vec::with_capacity(series_infos.len());

                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (series_poster, series_poster_command) =
                        SeriesPoster::new(index, series_info);
                    series_posters.push(series_poster);
                    series_posters_commands.push(series_poster_command);
                }
                self.series = series_posters;

                let series_posters_command =
                    Command::batch(series_posters_commands).map(|message| {
                        Message::SeriesPosterAction(message.get_id().unwrap_or(0), message)
                    });

                Command::batch([series_posters_command, upcoming_series_release_command])
            }
            Message::UpcomingReleasePosterAction(index, message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_info) = message {
                    return Command::perform(async {}, |_| Message::SeriesSelected(series_info));
                }
                return self.upcoming_releases[index]
                    .0
                    .update(message)
                    .map(move |message| Message::SeriesPosterAction(index, message));
            }
            Message::UpcomingReleaseSeriesReceived(upcoming_series) => {
                let mut upcoming_series_posters = Vec::with_capacity(upcoming_series.len());
                let mut upcoming_series_posters_commands =
                    Vec::with_capacity(upcoming_series.len());

                for (index, (series_info, episode_and_release_time)) in
                    upcoming_series.into_iter().enumerate()
                {
                    let (series_poster, series_poster_command) =
                        SeriesPoster::new(index, series_info);
                    upcoming_series_posters.push((series_poster, episode_and_release_time));
                    upcoming_series_posters_commands.push(series_poster_command);
                }

                self.upcoming_releases = upcoming_series_posters;

                Command::batch(upcoming_series_posters_commands).map(|message| {
                    Message::UpcomingReleasePosterAction(message.get_id().unwrap_or(0), message)
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
                let upcoming_series_releases = self
                    .upcoming_releases
                    .iter()
                    .enumerate()
                    .map(|(index, (series_poster, _))| {
                        series_poster
                            .release_series_posters_view(&self.upcoming_releases[index].1)
                            .map(|message| {
                                Message::UpcomingReleasePosterAction(
                                    message.get_id().unwrap_or(0),
                                    message,
                                )
                            })
                    })
                    .collect();

                let upcoming_series_releases = Column::with_children(upcoming_series_releases);

                let running_shows =
                    Wrap::with_elements(Self::filter_posters(&self.series, SeriesStatus::Running))
                        .spacing(5.0)
                        .padding(5.0);
                let ended_shows =
                    Wrap::with_elements(Self::filter_posters(&self.series, SeriesStatus::Ended))
                        .spacing(5.0)
                        .padding(5.0);

                let content = column!(
                    upcoming_series_releases,
                    text("Running").size(20).style(GREEN_THEME),
                    running_shows,
                    text("Ended").size(20).style(RED_THEME),
                    ended_shows
                )
                .spacing(5.0)
                .width(Length::Fill)
                .padding(5.0);

                scrollable(content).into()
            }
        }
    }

    fn filter_posters(
        posters: &Vec<SeriesPoster>,
        status: SeriesStatus,
    ) -> Vec<Element<'_, Message, Renderer>> {
        posters
            .iter()
            .filter(|poster| poster.get_status().unwrap() == status)
            .map(|poster| {
                poster.view().map(|message| {
                    Message::SeriesPosterAction(message.get_id().unwrap_or(0), message)
                })
            })
            .collect()
    }
}

/// Returns series info and their associated release episode and time
async fn get_series_release_time(
    series_informations: Vec<SeriesMainInformation>,
) -> Vec<(SeriesMainInformation, (Episode, String))> {
    let handles: Vec<_> = series_informations
        .iter()
        .map(|series_info| tokio::spawn(caching::episode_list::EpisodeList::new(series_info.id)))
        .collect();

    let mut episodes_lists = Vec::with_capacity(handles.len());
    for handle in handles {
        let episode_list = handle
            .await
            .expect("failed to await episode list handle")
            .expect("failed to get episode list");

        episodes_lists.push(episode_list);
    }

    series_informations
        .into_iter()
        .zip(episodes_lists.into_iter())
        .filter(|(series_info, _)| SeriesStatus::new(series_info) != SeriesStatus::Ended)
        .filter(|(_, episode_list)| episode_list.get_next_episode_and_time().is_some())
        .map(|(series_info, episode_list)| {
            (
                series_info.clone(),
                episode_list
                    .get_next_episode_and_time()
                    .map(|(episode, release_time)| (episode.clone(), release_time))
                    .unwrap(),
            )
        })
        .collect()
}

impl Tab for MyShowsTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "My Shows".to_owned()
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text("My Shows icon".to_owned())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::MyShows)
    }
}
