use std::sync::mpsc;

use iced::widget::{container, Column};
use iced::{Element, Length, Task};
use iced_aw::Spinner;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::caching;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::gui::message::IndexedMessage;
use crate::gui::tabs::tab_searching::{unavailable_posters, Searchable};
use crate::gui::{helpers, styles};
use upcoming_poster::{Message as UpcomingPosterMessage, UpcomingPoster};

#[derive(Debug, Clone)]
pub enum Message {
    UpcomingPoster(IndexedMessage<usize, UpcomingPosterMessage>),
    SeriesInformationReceived(Option<Vec<(SeriesMainInformation, Episode, EpisodeReleaseTime)>>),
    Refresh,
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct UpcomingReleases<'a> {
    load_state: LoadState,
    upcoming_posters: Vec<UpcomingPoster<'a>>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    /// A collection of matched series id after a fuzzy search
    matched_id_collection: Option<Vec<u32>>,
}

impl UpcomingReleases<'_> {
    pub fn new(series_page_sender: mpsc::Sender<SeriesMainInformation>) -> (Self, Task<Message>) {
        (
            Self {
                load_state: LoadState::default(),
                upcoming_posters: vec![],
                series_page_sender,
                matched_id_collection: None,
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
                    helpers::time::NaiveTime::new(num_minutes as u32).get_longest_unit_duration();

                if let Some(duration) = duration {
                    iced::time::every(std::time::Duration::from_secs(duration.num_seconds() as u64))
                        .map(|_| Message::Refresh)
                } else {
                    iced::time::every(std::time::Duration::from_secs(60)).map(|_| Message::Refresh)
                }
            })
            .unwrap_or(iced::Subscription::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
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
                        std::borrow::Cow::Owned(series_info),
                        self.series_page_sender.clone(),
                        episode,
                        release_time,
                    );
                    series_posters.push(poster);
                    series_posters_commands.push(command);
                }
                self.upcoming_posters = series_posters;
                Task::batch(series_posters_commands).map(Message::UpcomingPoster)
            }
            Message::UpcomingPoster(message) => self.upcoming_posters[message.index()]
                .update(message)
                .map(Message::UpcomingPoster),
            Message::Refresh => load_upcoming_releases(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if let LoadState::Loading = self.load_state {
            return container(Spinner::new())
                .center_x(Length::Fill)
                .center_y(100)
                .into();
        }
        if self.upcoming_posters.is_empty() {
            Self::empty_upcoming_posters()
        } else {
            let upcoming_posters: Vec<Element<'_, Message>> = self
                .upcoming_posters
                .iter()
                .filter(|poster| {
                    if let Some(matched_id_collection) = &self.matched_id_collection {
                        self.is_matched_id(matched_id_collection, poster.get_series_info().id)
                    } else {
                        true
                    }
                })
                .map(|poster| poster.view().map(Message::UpcomingPoster))
                .collect();

            if upcoming_posters.is_empty() {
                Self::no_search_matches()
            } else {
                Column::with_children(upcoming_posters)
                    .spacing(5)
                    .width(Length::Fill)
                    .align_x(iced::Alignment::Center)
                    .into()
            }
        }
    }

    fn empty_upcoming_posters() -> Element<'static, Message> {
        unavailable_posters("No Upcoming Episodes")
            .style(styles::container_styles::first_class_container_square_theme)
            .height(200)
            .width(Length::Fill)
            .into()
    }

    fn no_search_matches() -> Element<'static, Message> {
        unavailable_posters("No matches found!")
            .style(styles::container_styles::first_class_container_square_theme)
            .height(200)
            .width(Length::Fill)
            .into()
    }
}

impl Searchable for UpcomingReleases<'_> {
    fn get_series_information_collection(&self) -> Vec<&SeriesMainInformation> {
        self.upcoming_posters
            .iter()
            .map(|poster| poster.get_series_info())
            .collect()
    }

    fn matches_id_collection(&mut self) -> &mut Option<Vec<u32>> {
        &mut self.matched_id_collection
    }
}

fn load_upcoming_releases() -> Task<Message> {
    Task::perform(
        async {
            caching::series_list::SeriesList::new()
                .get_upcoming_release_series_information_and_episodes()
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

    use iced::widget::{column, container, horizontal_space, image, mouse_area, row, text, Space};
    use iced::{Element, Length, Task};

    #[derive(Clone, Debug)]
    pub enum Message {
        Poster(GenericPosterMessage),
        SeriesPosterPressed,
    }

    pub struct UpcomingPoster<'a> {
        index: usize,
        poster: GenericPoster<'a>,
        upcoming_episode: Episode,
        episode_release_time: EpisodeReleaseTime,
    }

    impl<'a> UpcomingPoster<'a> {
        pub fn new(
            index: usize,
            series_info: std::borrow::Cow<'a, SeriesMainInformation>,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
            upcoming_episode: Episode,
            episode_release_time: EpisodeReleaseTime,
        ) -> (Self, Task<IndexedMessage<usize, Message>>) {
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

        pub fn get_series_info(&self) -> &SeriesMainInformation {
            self.poster.get_series_info()
        }

        pub fn get_episode_release_time(&self) -> &EpisodeReleaseTime {
            &self.episode_release_time
        }

        pub fn update(
            &mut self,
            message: IndexedMessage<usize, Message>,
        ) -> Task<IndexedMessage<usize, Message>> {
            match message.message() {
                Message::Poster(message) => {
                    self.poster.update(message);
                    Task::none()
                }
                Message::SeriesPosterPressed => {
                    self.poster.open_series_page();
                    Task::none()
                }
            }
        }

        pub fn view(&self) -> Element<IndexedMessage<usize, Message>> {
            let mut content = row!().padding(2).spacing(7);
            if let Some(image_bytes) = self.poster.get_image() {
                let image_handle = image::Handle::from_bytes(image_bytes.clone());
                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(helpers::empty_image::empty_image().width(100).height(140));
            };

            let mut metadata = column!().spacing(5);
            metadata = metadata.push(
                text(self.poster.get_series_info().name.clone())
                    .size(18)
                    .style(styles::text_styles::accent_color_theme),
            );
            // Some separation between series name and the rest of content
            metadata = metadata.push(Space::with_height(10));

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

            metadata = metadata.push(text(self.episode_release_time.to_string()));

            content = content.push(metadata);

            content = content.push(horizontal_space());
            let release_time_widget = container(
                container(
                    helpers::time::NaiveTime::new(
                        self.episode_release_time
                            .get_remaining_release_duration()
                            .num_minutes() as u32,
                    )
                    .largest_part()
                    .map(|(time_value, time_text)| {
                        column![text(time_value), text(time_text),].align_x(iced::Alignment::Center)
                    })
                    .unwrap_or(column![text("Now")]),
                )
                .padding(5)
                .center_x(70)
                .center_y(70)
                .style(styles::container_styles::release_time_container_theme),
            )
            .center_x(Length::Shrink)
            .center_y(140);

            content = content.push(release_time_widget);

            let content = container(content)
                .padding(5)
                .style(styles::container_styles::first_class_container_rounded_theme)
                .width(1000);

            let element: Element<'_, Message> = mouse_area(content)
                .on_press(Message::SeriesPosterPressed)
                .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }
    }
}
