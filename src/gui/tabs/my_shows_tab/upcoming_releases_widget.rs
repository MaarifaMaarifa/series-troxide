use std::sync::mpsc;

use iced::widget::{container, text, Column};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

use crate::core::api::episodes_information::Episode;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::caching;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::gui::{helpers, styles};

#[derive(Debug, Clone)]
pub enum Message {
    SeriesPosters(SeriesPosterMessage),
    SeriesInformationReceived(Option<Vec<(SeriesMainInformation, Episode, EpisodeReleaseTime)>>),
    Refresh,
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct UpcomingReleases {
    load_state: LoadState,
    series_posters: Vec<(SeriesPoster, Episode, EpisodeReleaseTime)>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
}

impl UpcomingReleases {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                load_state: LoadState::default(),
                series_posters: vec![],
                series_page_sender,
            },
            load_upcoming_releases(),
        )
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Refreshing the widget so as to avoid having outdated
        // episodes release time
        self.series_posters
            .first()
            .map(|(_, _, episode_release_time)| {
                let num_minutes = episode_release_time
                    .get_remaining_release_duration()
                    .num_minutes();
                let duration =
                    helpers::time::SaneTime::new(num_minutes as u32).get_longest_unit_duration();

                if let Some(duration) = duration {
                    iced::time::every(std::time::Duration::from_secs(duration.num_seconds() as u64))
                        .map(|_| Message::Refresh)
                } else {
                    iced::time::every(std::time::Duration::from_secs(60)).map(|_| Message::Refresh)
                }
            })
            .unwrap_or(iced::Subscription::none())
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesInformationReceived(series_infos) => {
                self.load_state = LoadState::Loaded;
                let mut series_infos = series_infos.unwrap();

                // sorting the list according to release time
                series_infos.sort_by_key(|(_, _, release_time)| release_time.clone());

                let mut series_posters_commands = Vec::with_capacity(series_infos.len());
                let mut series_posters = Vec::with_capacity(series_infos.len());

                for (index, (series_info, episode, release_time)) in
                    series_infos.into_iter().enumerate()
                {
                    let (poster, command) = SeriesPoster::new(index, series_info);
                    series_posters.push((poster, episode, release_time));
                    series_posters_commands.push(command);
                }
                self.series_posters = series_posters;
                Command::batch(series_posters_commands).map(Message::SeriesPosters)
            }
            Message::SeriesPosters(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_info) = message.clone() {
                    self.series_page_sender
                        .send(*series_info)
                        .expect("failed to send the series page");
                    return Command::none();
                }
                self.series_posters[message.get_index().expect("message should have and index")]
                    .0
                    .update(message)
                    .map(Message::SeriesPosters)
            }
            Message::Refresh => load_upcoming_releases(),
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
                .style(styles::container_styles::first_class_container_square_theme())
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
                    .map(|(index, (poster, _, _))| {
                        poster
                            .release_series_posters_view({
                                let (_, episode, release_time) = &self.series_posters[index];
                                (episode, release_time)
                            })
                            .map(Message::SeriesPosters)
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

fn load_upcoming_releases() -> Command<Message> {
    Command::perform(
        async {
            caching::series_list::SeriesList::new()
                .get_upcoming_release_series_informations_and_episodes()
                .await
        },
        |res| Message::SeriesInformationReceived(res.ok()),
    )
}
