use std::sync::mpsc;

use iced::widget::{container, text, Column};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::caching;
use crate::core::caching::episode_list::EpisodeList;
use crate::gui::styles;
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::gui::view::series_view;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesPosters(usize, SeriesPosterMessage),
    SeriesInformationReceived(Option<Vec<(SeriesMainInformation, EpisodeList)>>),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct UpcomingReleases {
    load_state: LoadState,
    series_posters: Vec<(SeriesPoster, EpisodeList)>,
    series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
}

impl UpcomingReleases {
    pub fn new(
        series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                load_state: LoadState::default(),
                series_posters: vec![],
                series_page_sender,
            },
            Command::perform(
                async {
                    caching::series_list::SeriesList::new()
                        .get_upcoming_release_series_informations_and_episodes()
                        .await
                },
                |res| Message::SeriesInformationReceived(res.ok()),
            ),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesInformationReceived(series_infos) => {
                self.load_state = LoadState::Loaded;
                let mut series_infos = series_infos.unwrap();

                // sorting the list according to release time
                series_infos.sort_by_key(|(_, episode_list)| {
                    episode_list.get_next_episode_and_time().unwrap().1
                });

                let mut series_posters_commands = Vec::with_capacity(series_infos.len());
                let mut series_posters = Vec::with_capacity(series_infos.len());

                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (poster, command) = SeriesPoster::new(index, series_info.0);
                    series_posters.push((poster, series_info.1));
                    series_posters_commands.push(command);
                }
                self.series_posters = series_posters;
                Command::batch(series_posters_commands)
                    .map(|message| Message::SeriesPosters(message.get_id().unwrap_or(0), message))
            }
            Message::SeriesPosters(index, message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_info) = message.clone() {
                    self.series_page_sender
                        .send(series_view::Series::from_series_information(*series_info))
                        .expect("failed to send the series page");
                    return Command::none();
                }
                self.series_posters[index]
                    .0
                    .update(message)
                    .map(|message| Message::SeriesPosters(message.get_id().unwrap_or(0), message))
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        if let LoadState::Loading = self.load_state {
            return container(Spinner::new())
                .center_x()
                .center_y()
                .height(100)
                .width(Length::Fill)
                .into();
        }
        if self.series_posters.is_empty() {
            container(text("No Upcoming Episodes"))
                .style(styles::container_styles::first_class_container_tab_theme())
                .center_x()
                .center_y()
                .height(100)
                .width(Length::Fill)
                .into()
        } else {
            Column::with_children(
                self.series_posters
                    .iter()
                    .enumerate()
                    .map(|(index, (poster, _))| {
                        poster
                            .release_series_posters_view(
                                self.series_posters[index]
                                    .1
                                    .get_next_episode_and_time()
                                    .unwrap(),
                            )
                            .map(|message| {
                                Message::SeriesPosters(message.get_id().unwrap_or(0), message)
                            })
                    })
                    .collect(),
            )
            .spacing(5)
            .width(Length::Fill)
            .align_items(iced::Alignment::Center)
            .into()
        }
    }
}
