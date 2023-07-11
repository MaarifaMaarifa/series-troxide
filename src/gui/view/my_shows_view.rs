use std::collections::HashSet;

use crate::core::api::episodes_information::Episode;
use crate::core::caching;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::core::{api::series_information::SeriesMainInformation, database};
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::gui::troxide_widget::{GREEN_THEME, RED_THEME};
use crate::gui::{Message as GuiMessage, Tab};
use iced::widget::{container, scrollable, Column};
use iced_aw::{Spinner, Wrap};

use iced::{
    widget::{column, text},
    Command, Element, Renderer,
};
use iced::{Alignment, Length};

use super::series_view::SeriesStatus;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationsReceived(Vec<SeriesMainInformation>),
    SeriesSelected(Box<SeriesMainInformation>),
    SeriesPosterAction(usize, SeriesPosterMessage),
    UpcomingReleaseSeriesReceived(Vec<(SeriesMainInformation, (Episode, EpisodeReleaseTime))>),
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
    upcoming_releases: Vec<(SeriesPoster, (Episode, EpisodeReleaseTime))>,
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
            Message::UpcomingReleaseSeriesReceived(mut upcoming_series) => {
                // Sorting the upcoming series by release time
                upcoming_series.sort_by(|(_, (_, release_time_a)), (_, (_, release_time_b))| {
                    release_time_a.cmp(release_time_b)
                });

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
                    .filter(|(_, (poster, _))| {
                        poster
                            .get_series_information()
                            .map(|series_info| {
                                database::DB
                                    .get_series(series_info.id)
                                    .map(|series| series.is_tracked())
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false)
                    })
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

                let upcoming_series_releases = Column::with_children(upcoming_series_releases)
                    .spacing(5)
                    .align_items(Alignment::Center)
                    .width(Length::Fill)
                    .padding(10);

                let upcoming_posters = self
                    .upcoming_releases
                    .iter()
                    .map(|(poster, _)| poster)
                    .collect();

                let running_shows = Wrap::with_elements(Self::filter_posters(
                    &self.series,
                    &upcoming_posters,
                    MyShowsFilter::WaitingRelease,
                ))
                .spacing(5.0)
                .line_spacing(5.0)
                .padding(5.0);

                let ended_shows = Wrap::with_elements(Self::filter_posters(
                    &self.series,
                    &upcoming_posters,
                    MyShowsFilter::Ended,
                ))
                .spacing(5.0)
                .line_spacing(5.0)
                .padding(5.0);

                let untracked_shows = Wrap::with_elements(Self::filter_posters(
                    &self.series,
                    &upcoming_posters,
                    MyShowsFilter::Untracked,
                ))
                .spacing(5.0)
                .line_spacing(5.0)
                .padding(5.0);

                let content = column!(
                    upcoming_series_releases,
                    text("Waiting For Release Date").size(20).style(GREEN_THEME),
                    running_shows,
                    text("Ended").size(20).style(RED_THEME),
                    ended_shows,
                    text("Untracked").size(20),
                    untracked_shows
                )
                .spacing(5.0)
                .width(Length::Fill)
                .padding(5.0);

                scrollable(content).into()
            }
        }
    }

    fn filter_posters<'a>(
        posters: &'a Vec<SeriesPoster>,
        upcoming_series_posters: &Vec<&'a SeriesPoster>,
        filter: MyShowsFilter,
    ) -> Vec<Element<'a, Message, Renderer>> {
        match filter {
            MyShowsFilter::WaitingRelease => {
                let all_posters: HashSet<&SeriesPoster> = posters.iter().collect();
                let ended_posters: HashSet<&SeriesPoster> =
                    Self::get_ended_posters(posters).into_iter().collect();
                let upcoming_series_posters: HashSet<&SeriesPoster> = upcoming_series_posters
                    .into_iter()
                    .map(|poster| *poster)
                    .collect();

                let diff: HashSet<&SeriesPoster> = all_posters
                    .difference(&ended_posters)
                    .map(|poster| *poster)
                    .collect();

                let untracked_posters: HashSet<_> =
                    Self::get_untracked_posters(posters).into_iter().collect();

                let diff: HashSet<_> = diff
                    .difference(&untracked_posters)
                    .into_iter()
                    .map(|poster| *poster)
                    .collect();
                /*
                The expected series posters obtained from the final set difference operation will have
                a valid message id that can be used in the posters field in the MyShows struct
                */
                diff.difference(&upcoming_series_posters)
                    .map(|poster| {
                        poster.view().map(|message| {
                            Message::SeriesPosterAction(message.get_id().unwrap_or(0), message)
                        })
                    })
                    .collect()
            }
            MyShowsFilter::Ended => {
                let ended_posters: HashSet<_> =
                    Self::get_ended_posters(posters).into_iter().collect();
                let untracked_posters: HashSet<_> =
                    Self::get_untracked_posters(posters).into_iter().collect();
                let ended_posters: Vec<_> = ended_posters
                    .difference(&untracked_posters)
                    .into_iter()
                    .collect();

                ended_posters
                    .iter()
                    .map(|poster| {
                        poster.view().map(|message| {
                            Message::SeriesPosterAction(message.get_id().unwrap_or(0), message)
                        })
                    })
                    .collect()
            }
            MyShowsFilter::Untracked => {
                let posters = Self::get_untracked_posters(posters);
                posters
                    .iter()
                    .map(|poster| {
                        poster.view().map(|message| {
                            Message::SeriesPosterAction(message.get_id().unwrap_or(0), message)
                        })
                    })
                    .collect()
            }
        }
    }

    fn get_ended_posters(posters: &Vec<SeriesPoster>) -> Vec<&SeriesPoster> {
        posters
            .iter()
            .filter(|poster| poster.get_status().unwrap() == SeriesStatus::Ended)
            .collect()
    }

    fn get_untracked_posters(posters: &Vec<SeriesPoster>) -> Vec<&SeriesPoster> {
        posters
            .iter()
            .filter(|poster| {
                poster
                    .get_series_information()
                    .map(|series_info| {
                        database::DB
                            .get_series(series_info.id)
                            .map(|series| !series.is_tracked())
                            .unwrap_or(true)
                    })
                    .unwrap_or(true)
            })
            .collect()
    }
}

/// Returns series info and their associated release episode and time
async fn get_series_release_time(
    series_informations: Vec<SeriesMainInformation>,
) -> Vec<(SeriesMainInformation, (Episode, EpisodeReleaseTime))> {
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
        iced_aw::TabLabel::Text(self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::MyShows)
    }
}

enum MyShowsFilter {
    WaitingRelease,
    Untracked,
    Ended,
}
