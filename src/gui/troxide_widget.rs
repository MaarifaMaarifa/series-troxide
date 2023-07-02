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
    use crate::core::{caching, database};
    use crate::gui::view::series_view::SeriesStatus;
    use iced::widget::{column, container, image, mouse_area, progress_bar, row, text};
    use iced::{Command, Element, Renderer};

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
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                );

                mouse_area(content)
                    .on_press(Message::SeriesPosterPressed(Box::new(series_info.clone())))
                    .into()
            } else {
                container("").into()
            }
        }

        /// View intended for the watchlist tab
        pub fn watchlist_view(&self, total_episodes: f32) -> Element<'_, Message, Renderer> {
            let mut content = row!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            };

            let mut metadata = column!().padding(2).spacing(1);

            if let Some(series_info) = &self.series_information {
                metadata = metadata.push(text(&series_info.name));

                let watched_episodes = database::DB
                    .get_series(series_info.id)
                    .unwrap()
                    .get_total_episodes_watched() as f32;

                let progress_bar = row!(
                    progress_bar(0.0..=total_episodes, watched_episodes,)
                        .height(10)
                        .width(500),
                    text(format!("{}/{}", watched_episodes, total_episodes))
                )
                .spacing(5);

                metadata = metadata.push(progress_bar);

                let last_episode_watched = if let Some((season_num, last_watched_season)) =
                    database::DB
                        .get_series(series_info.id)
                        .unwrap()
                        .get_last_season()
                {
                    last_watched_season.get_last_episode();
                    text(format!(
                        "Last Watched Episode: S{}E{}",
                        parse_season_episode_number(season_num),
                        parse_season_episode_number(last_watched_season
                            .get_last_episode()
                            .expect("the season should have atleast one episode for it to be the last watched"))
                    ))
                } else {
                    text("No Episode Watched")
                };

                metadata = metadata.push(last_episode_watched);

                content = content.push(metadata);

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

    fn parse_season_episode_number(number: u32) -> String {
        if number < 10_u32 {
            format!("0{}", number)
        } else {
            number.to_string()
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
