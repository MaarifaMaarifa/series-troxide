use std::sync::mpsc;

use iced::widget::{container, text, Column};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::caching;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::gui::message::IndexedMessage;
use crate::gui::{helpers, styles};
use upcoming_poster::{Message as UpcomingPosterMessage, UpcomingPoster};

#[derive(Debug, Clone)]
pub enum Message {
    UpcomingPoster(IndexedMessage<UpcomingPosterMessage>),
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
    upcoming_posters: Vec<UpcomingPoster>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
}

impl UpcomingReleases {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                load_state: LoadState::default(),
                upcoming_posters: vec![],
                series_page_sender,
            },
            load_upcoming_releases(),
        )
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Refreshing the widget so as to avoid having outdated
        // episodes release time
        self.upcoming_posters
            .first()
            .map(|poster| {
                let num_minutes = poster
                    .get_episode_release_time()
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
                    let (poster, command) = UpcomingPoster::new(
                        index,
                        series_info,
                        self.series_page_sender.clone(),
                        episode,
                        release_time,
                    );
                    series_posters.push(poster);
                    series_posters_commands.push(command);
                }
                self.upcoming_posters = series_posters;
                Command::batch(series_posters_commands).map(Message::UpcomingPoster)
            }
            Message::UpcomingPoster(message) => self.upcoming_posters[message.index()]
                .update(message)
                .map(Message::UpcomingPoster),
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
        if self.upcoming_posters.is_empty() {
            container(text("No Upcoming Episodes"))
                .style(styles::container_styles::first_class_container_square_theme())
                .center_x()
                .center_y()
                .height(100)
                .width(Length::Fill)
                .into()
        } else {
            Column::with_children(
                self.upcoming_posters
                    .iter()
                    .map(|poster| poster.view().map(Message::UpcomingPoster))
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

mod upcoming_poster {
    use std::sync::mpsc;

    use crate::core::api::tv_maze::episodes_information::Episode;
    use crate::core::{
        api::tv_maze::series_information::SeriesMainInformation,
        caching::episode_list::EpisodeReleaseTime,
    };
    use crate::gui::helpers::{self, season_episode_str_gen};
    use crate::gui::message::IndexedMessage;
    use crate::gui::styles;
    use crate::gui::troxide_widget::series_poster::{GenericPoster, GenericPosterMessage};

    use iced::widget::{
        column, container, horizontal_space, image, mouse_area, row, text, vertical_space, Space,
    };
    use iced::{Command, Element, Length, Renderer};

    #[derive(Clone, Debug)]
    pub enum Message {
        Poster(GenericPosterMessage),
        SeriesPosterPressed,
    }
    pub struct UpcomingPoster {
        index: usize,
        poster: GenericPoster,
        upcoming_episode: Episode,
        episode_release_time: EpisodeReleaseTime,
    }

    impl UpcomingPoster {
        pub fn new(
            index: usize,
            series_info: SeriesMainInformation,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
            upcoming_episode: Episode,
            episode_release_time: EpisodeReleaseTime,
        ) -> (Self, Command<IndexedMessage<Message>>) {
            let (poster, poster_command) = GenericPoster::new(series_info, series_page_sender);
            (
                Self {
                    index,
                    poster,
                    upcoming_episode,
                    episode_release_time,
                },
                poster_command
                    .map(Message::Poster)
                    .map(move |message| IndexedMessage::new(index, message)),
            )
        }

        pub fn get_episode_release_time(&self) -> &EpisodeReleaseTime {
            &self.episode_release_time
        }

        pub fn update(
            &mut self,
            message: IndexedMessage<Message>,
        ) -> Command<IndexedMessage<Message>> {
            match message.message() {
                Message::Poster(message) => {
                    self.poster.update(message);
                    Command::none()
                }
                Message::SeriesPosterPressed => {
                    self.poster.open_series_page();
                    Command::none()
                }
            }
        }

        pub fn view(&self) -> Element<'_, IndexedMessage<Message>, Renderer> {
            let mut content = row!().padding(2).spacing(7);
            if let Some(image_bytes) = self.poster.get_image() {
                let image_handle = image::Handle::from_memory(image_bytes.clone());
                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(Space::new(100, 140));
            };

            let mut metadata = column!().spacing(5);
            metadata = metadata.push(
                text(&self.poster.get_series_info().name)
                    .size(18)
                    .style(styles::text_styles::accent_color_theme()),
            );
            // Some separation between series name and the rest of content
            metadata = metadata.push(vertical_space(10));

            let season_number = self.upcoming_episode.season;
            let episode_number = self
                .upcoming_episode
                .number
                .expect("an episode should have a valid number");

            let episode_name = &self.upcoming_episode.name;

            metadata = metadata.push(text(format!(
                "{}: {}",
                season_episode_str_gen(season_number, episode_number),
                episode_name,
            )));

            metadata = metadata.push(text(
                self.episode_release_time.get_full_release_date_and_time(),
            ));

            content = content.push(metadata);

            content = content.push(horizontal_space(Length::Fill));
            let release_time_widget = container(
                container(
                    helpers::time::SaneTime::new(
                        self.episode_release_time
                            .get_remaining_release_duration()
                            .num_minutes() as u32,
                    )
                    .get_time_plurized()
                    .into_iter()
                    .last()
                    .map(|(time_text, time_value)| {
                        column![text(time_value), text(time_text),]
                            .align_items(iced::Alignment::Center)
                    })
                    .unwrap_or(column![text("Now")]),
                )
                .width(70)
                .height(70)
                .padding(5)
                .center_x()
                .center_y()
                .style(styles::container_styles::release_time_container_theme()),
            )
            .center_x()
            .center_y()
            .height(140);

            content = content.push(release_time_widget);

            let content = container(content)
                .padding(5)
                .style(styles::container_styles::first_class_container_rounded_theme())
                .width(1000);

            let element: Element<'_, Message, Renderer> = mouse_area(content)
                .on_press(Message::SeriesPosterPressed)
                .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }
    }
}
