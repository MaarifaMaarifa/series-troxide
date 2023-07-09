// The text size of the beginning part of a info
pub const INFO_HEADER: u16 = 18;
// The text size of the main part of a info
pub const INFO_BODY: u16 = 15;

// const INFO_BODY_HEIGHT: u16 = INFO_HEADER - (INFO_HEADER - INFO_BODY);

const RED_COLOR: iced::Color = iced::Color::from_rgb(2.55, 0.0, 0.0);
const GREEN_COLOR: iced::Color = iced::Color::from_rgb(0.0, 1.28, 0.0);

pub const RED_THEME: iced::theme::Text = iced::theme::Text::Color(RED_COLOR);
pub const GREEN_THEME: iced::theme::Text = iced::theme::Text::Color(GREEN_COLOR);

pub mod series_poster {

    use crate::core::api::episodes_information::Episode;
    use crate::core::api::series_information::SeriesMainInformation;
    use crate::core::api::{get_series_from_episode, Image};
    use crate::core::caching::episode_list::EpisodeReleaseTime;
    use crate::core::{caching, database};
    use crate::gui::helpers::season_episode_str_gen;
    use crate::gui::styles;
    use crate::gui::view::series_view::SeriesStatus;
    use iced::widget::{
        column, container, horizontal_space, image, mouse_area, progress_bar, row, text,
        vertical_space,
    };
    use iced::{theme, Command, Element, Length, Renderer};

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(usize, Option<Vec<u8>>),
        SeriesInfoReceived(usize, SeriesMainInformation),
        SeriesPosterPressed(Box<SeriesMainInformation>),
    }

    impl Message {
        pub fn get_id(&self) -> Option<usize> {
            if let Self::ImageLoaded(id, _) = self {
                return Some(id.to_owned());
            }
            if let Self::SeriesInfoReceived(id, _) = self {
                return Some(id.to_owned());
            }
            None
        }
    }

    #[derive(PartialEq, Eq, Hash)]
    pub struct SeriesPoster {
        series_information: Option<SeriesMainInformation>,
        image: Option<Vec<u8>>,
    }

    impl SeriesPoster {
        pub fn new(
            id: usize,
            series_information: SeriesMainInformation,
        ) -> (Self, Command<Message>) {
            let image_url = series_information.image.clone();

            let poster = Self {
                series_information: Some(series_information),
                image: None,
            };

            let series_image_command = poster_image_command(id, image_url);

            (poster, series_image_command)
        }

        pub fn from_episode_info(id: usize, episode_info: Episode) -> (Self, Command<Message>) {
            let poster = Self {
                series_information: None,
                image: None,
            };

            let command =
                Command::perform(get_series_from_episode(episode_info), move |series_info| {
                    Message::SeriesInfoReceived(
                        id,
                        series_info.expect("failed to get series information"),
                    )
                });
            (poster, command)
        }

        pub fn update(&mut self, message: Message) -> Command<Message> {
            match message {
                Message::ImageLoaded(_, image) => self.image = image,
                Message::SeriesPosterPressed(_) => {
                    unimplemented!("the series poster should not handle being pressed")
                }
                Message::SeriesInfoReceived(id, series_info) => {
                    let image_url = series_info.image.clone();
                    self.series_information = Some(series_info);
                    return poster_image_command(id, image_url);
                }
            }
            Command::none()
        }

        /// Views the series poster widget
        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            };

            if let Some(series_info) = &self.series_information {
                content = content.push(
                    text(&series_info.name)
                        .size(15)
                        .width(100)
                        .height(30)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                );

                let content = container(content)
                    .padding(5)
                    .style(theme::Container::Custom(Box::new(
                        styles::container_styles::ContainerThemeSecond,
                    )
                        as Box<dyn container::StyleSheet<Style = iced::Theme>>));

                mouse_area(content)
                    .on_press(Message::SeriesPosterPressed(Box::new(series_info.clone())))
                    .into()
            } else {
                container("").into()
            }
        }

        /// View intended for the watchlist tab
        pub fn watchlist_view(&self, total_episodes: usize) -> Element<'_, Message, Renderer> {
            let mut content = row!().padding(2).spacing(5);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            };

            let mut metadata = column!().padding(2).spacing(5);

            if let Some(series_info) = &self.series_information {
                metadata = metadata.push(text(&series_info.name));
                metadata = metadata.push(vertical_space(10));

                let watched_episodes = database::DB
                    .get_series(series_info.id)
                    .map(|series| series.get_total_episodes())
                    .unwrap_or(0);

                let last_episode_watched = if let Some(series) =
                    database::DB.get_series(series_info.id)
                {
                    if let Some((season_num, last_watched_season)) = series.get_last_season() {
                        last_watched_season.get_last_episode();
                        text(format!("{} {}","Last watched episode", season_episode_str_gen(season_num, last_watched_season.get_last_episode().expect("the season should have atleast one episode for it to be the last watched"))))
                    } else {
                        text("No Episode Watched")
                    }
                } else {
                    text("No Episode Watched")
                };

                metadata = metadata.push(last_episode_watched);

                let progress_bar = row!(
                    progress_bar(0.0..=total_episodes as f32, watched_episodes as f32,)
                        .height(10)
                        .width(500),
                    text(format!(
                        "{}/{}",
                        watched_episodes as f32, total_episodes as f32
                    ))
                )
                .spacing(5);

                metadata = metadata.push(progress_bar);

                let episodes_left = total_episodes - watched_episodes;

                metadata = metadata.push(text(format!("{} episodes left", episodes_left)));

                content = content.push(metadata);

                let content = container(content)
                    .padding(5)
                    .style(theme::Container::Custom(Box::new(
                        styles::container_styles::ContainerThemeFirst,
                    )
                        as Box<dyn container::StyleSheet<Style = iced::Theme>>))
                    .width(1000);

                mouse_area(content)
                    .on_press(Message::SeriesPosterPressed(Box::new(series_info.clone())))
                    .into()
            } else {
                container("").into()
            }
        }

        pub fn release_series_posters_view(
            &self,
            episode_and_release_time: &(Episode, EpisodeReleaseTime),
        ) -> Element<'_, Message, Renderer> {
            let mut content = row!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            };

            let mut metadata = column!();
            if let Some(series_info) = &self.series_information {
                metadata = metadata.push(text(&series_info.name));

                let season_number = episode_and_release_time.0.season;
                let episode_number = episode_and_release_time
                    .0
                    .number
                    .expect("an episode should have a valid number");

                let episode_name = &episode_and_release_time.0.name;

                metadata = metadata.push(text(format!(
                    "{}: {}",
                    season_episode_str_gen(season_number, episode_number),
                    episode_name,
                )));

                metadata = metadata.push(text(
                    episode_and_release_time.1.get_full_release_date_and_time(),
                ));

                content = content.push(metadata);

                content = content.push(horizontal_space(Length::Fill));
                let release_time_widget = container(
                    container(
                        text(
                            &episode_and_release_time
                                .1
                                .get_remaining_release_time()
                                .unwrap(),
                        )
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    )
                    .width(70)
                    .height(70)
                    .padding(5)
                    .center_x()
                    .center_y()
                    .style(theme::Container::Custom(Box::new(
                        styles::container_styles::ContainerThemeReleaseTime,
                    )
                        as Box<dyn container::StyleSheet<Style = iced::Theme>>)),
                )
                .center_x()
                .center_y()
                .height(140);

                content = content.push(release_time_widget);

                let content = container(content)
                    .padding(5)
                    .style(theme::Container::Custom(Box::new(
                        styles::container_styles::ContainerThemeFirst,
                    )
                        as Box<dyn container::StyleSheet<Style = iced::Theme>>))
                    .width(1000);

                mouse_area(content)
                    .on_press(Message::SeriesPosterPressed(Box::new(series_info.clone())))
                    .into()
            } else {
                container("").into()
            }
        }

        pub fn get_status(&self) -> Option<SeriesStatus> {
            self.series_information
                .as_ref()
                .map(|series_info| SeriesStatus::new(&series_info))
        }
    }

    fn poster_image_command(id: usize, image: Option<Image>) -> Command<Message> {
        if let Some(image) = image {
            Command::perform(
                async move { caching::load_image(image.medium_image_url).await },
                move |image| Message::ImageLoaded(id, image),
            )
        } else {
            Command::none()
        }
    }
}
