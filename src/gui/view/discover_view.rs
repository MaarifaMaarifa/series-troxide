use crate::core::api::episodes_information::Episode;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::tv_schedule::get_episodes_with_date;
use crate::core::api::updates::show_updates::*;
use crate::gui::Message as GuiMessage;
use episode_poster::Message as EpisodePosterMessage;
use series_updates_poster::Message as SeriesPosterMessage;

use iced::{
    widget::scrollable::Properties,
    widget::{column, row, scrollable, text, Row},
    Command, Element, Length, Renderer,
};

#[derive(Default, PartialEq)]
enum LoadState {
    #[default]
    Waiting,
    Loading,
    Loaded,
}

#[derive(Clone, Debug)]
pub enum Message {
    LoadSchedule,
    ScheduleLoaded(Vec<Episode>),
    SeriesUpdatesLoaded(Vec<SeriesMainInformation>),
    EpisodePosterAction(/*episode poster index*/ usize, EpisodePosterMessage),
    SeriesPosterAction(/*series poster index*/ usize, SeriesPosterMessage),
    SeriesSelected(/*series_id*/ u32),
}

#[derive(Default)]
pub struct Discover {
    load_state: LoadState,
    new_episodes: Vec<episode_poster::EpisodePoster>,
    series_updates: Vec<series_updates_poster::SeriesPoster>,
}

impl Discover {
    pub fn update(&mut self, message: Message) -> Command<GuiMessage> {
        match message {
            Message::LoadSchedule => {
                if self.load_state != LoadState::Waiting {
                    return Command::none();
                }
                self.load_state = LoadState::Loading;

                let series_updates_command =
                    Command::perform(get_show_updates(UpdateTimestamp::Day, Some(10)), |series| {
                        GuiMessage::DiscoverAction(Message::SeriesUpdatesLoaded(
                            series.expect("Failed to load series updates"),
                        ))
                    });

                let new_episodes_command =
                    Command::perform(get_episodes_with_date(None), |episodes| {
                        GuiMessage::DiscoverAction(Message::ScheduleLoaded(
                            episodes.expect("Failed to load episodes schedule"),
                        ))
                    });

                Command::batch([series_updates_command, new_episodes_command])
            }
            Message::ScheduleLoaded(episodes) => {
                self.load_state = LoadState::Loaded;

                let mut episode_posters = Vec::with_capacity(episodes.len());
                let mut commands = Vec::with_capacity(episodes.len());
                for (index, episode) in episodes.into_iter().enumerate() {
                    let (poster, command) = episode_poster::EpisodePoster::new(index, episode);
                    episode_posters.push(poster);
                    commands.push(command);
                }

                self.new_episodes = episode_posters;
                return Command::batch(commands).map(|m| GuiMessage::DiscoverAction(m));
            }
            Message::EpisodePosterAction(index, message) => {
                return self.new_episodes[index]
                    .update(message)
                    .map(GuiMessage::DiscoverAction)
            }
            Message::SeriesUpdatesLoaded(series) => {
                let mut series_infos = Vec::with_capacity(series.len());
                let mut series_poster_commands = Vec::with_capacity(series.len());
                for (index, series_info) in series.into_iter().enumerate() {
                    let (series_poster, series_poster_command) =
                        series_updates_poster::SeriesPoster::new(index, series_info);
                    series_infos.push(series_poster);
                    series_poster_commands.push(series_poster_command);
                }
                self.series_updates = series_infos;

                Command::batch(series_poster_commands).map(GuiMessage::DiscoverAction)
            }
            Message::SeriesPosterAction(index, message) => {
                return self.series_updates[index]
                    .update(message)
                    .map(GuiMessage::DiscoverAction)
            }
            Message::SeriesSelected(_) => {
                unreachable!("Discover View should not handle Series View")
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        match self.load_state {
            LoadState::Loading => row!(text("loading..."))
                .align_items(iced::Alignment::Center)
                .width(Length::Fill)
                .into(),
            LoadState::Loaded => column!(load_new_episodes(self), load_series_updates(self)).into(),
            LoadState::Waiting => unreachable!(
                "the Waiting state should be changed when discover view is first viewed"
            ),
        }
    }
}

fn load_new_episodes(discover_view: &Discover) -> Element<'_, Message, Renderer> {
    scrollable(Row::with_children(
        discover_view
            .new_episodes
            .iter()
            .enumerate()
            .map(|(index, poster)| {
                poster
                    .view()
                    .map(move |m| Message::EpisodePosterAction(index, m))
            })
            .collect(),
    ))
    .width(Length::Fill)
    .horizontal_scroll(Properties::new().width(0).margin(0).scroller_width(0))
    .into()
}

fn load_series_updates(discover_view: &Discover) -> Element<'_, Message, Renderer> {
    scrollable(Row::with_children(
        discover_view
            .series_updates
            .iter()
            .enumerate()
            .map(|(index, poster)| {
                poster
                    .view()
                    .map(move |m| Message::SeriesPosterAction(index, m))
            })
            .collect(),
    ))
    .width(Length::Fill)
    .horizontal_scroll(Properties::new().width(0).margin(0).scroller_width(0))
    .into()
}

mod episode_poster {

    use crate::core::api::load_image;
    use crate::core::api::series_information::get_series_main_info_with_url;
    use crate::core::api::series_information::SeriesMainInformation;
    use iced::widget::mouse_area;
    use iced::widget::{column, image, text};
    use iced::{Command, Element, Renderer};

    use super::Episode;
    use super::Message as DiscoverMessage;

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(Option<Vec<u8>>),
        SeriesInformationLoaded(SeriesMainInformation),
        EpisodePosterPressed(/*series_id*/ u32),
    }

    pub struct EpisodePoster {
        index: usize,
        //episode: Episode,
        image: Option<Vec<u8>>,
        series_belonging: Option<SeriesMainInformation>,
    }

    impl EpisodePoster {
        pub fn new(index: usize, episode: Episode) -> (Self, Command<DiscoverMessage>) {
            let poster = Self {
                index,
                image: None,
                series_belonging: None,
            };

            let series_information_command = Command::perform(
                async move {
                    get_series_main_info_with_url(episode.links.show.href)
                        .await
                        .expect("could not obtain series information")
                },
                move |series| {
                    DiscoverMessage::EpisodePosterAction(
                        index,
                        Message::SeriesInformationLoaded(series),
                    )
                },
            );

            (poster, series_information_command)
        }

        pub fn update(&mut self, message: Message) -> Command<DiscoverMessage> {
            match message {
                Message::ImageLoaded(image) => self.image = image,
                Message::SeriesInformationLoaded(series_info) => {
                    let series_image_url = series_info.image.clone();
                    let poster_index = self.index;
                    self.series_belonging = Some(series_info);

                    if let Some(image) = series_image_url {
                        return Command::perform(
                            load_image(image.medium_image_url),
                            move |image| {
                                DiscoverMessage::EpisodePosterAction(
                                    poster_index,
                                    Message::ImageLoaded(image),
                                )
                            },
                        );
                    }
                }
                Message::EpisodePosterPressed(series_id) => {
                    return Command::perform(async {}, move |_| {
                        DiscoverMessage::SeriesSelected(series_id)
                    })
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).height(120);
                content = content.push(image);
            };

            if let Some(series_info) = &self.series_belonging {
                content = content.push(text(&series_info.name).size(15))
            }

            // content.push(text(&self.episode.name)).into()
            if let Some(series_info) = &self.series_belonging {
                mouse_area(content)
                    .on_press(Message::EpisodePosterPressed(series_info.id))
                    .into()
            } else {
                content.into()
            }
        }
    }
}

mod series_updates_poster {

    use crate::core::api::load_image;
    use crate::core::api::series_information::SeriesMainInformation;
    use iced::widget::{column, image, mouse_area, text};
    use iced::{Command, Element, Renderer};

    use super::Message as DiscoverMessage;

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(Option<Vec<u8>>),
        SeriesPosterPressed(/*series_id*/ u32),
    }

    pub struct SeriesPoster {
        //index: usize,
        series_information: SeriesMainInformation,
        image: Option<Vec<u8>>,
    }

    impl SeriesPoster {
        pub fn new(
            index: usize,
            series_information: SeriesMainInformation,
        ) -> (Self, Command<DiscoverMessage>) {
            let image_url = series_information.image.clone();

            let poster = Self {
                series_information,
                image: None,
            };

            let series_image_command = if let Some(image) = image_url {
                Command::perform(
                    async move { load_image(image.medium_image_url).await },
                    move |image| {
                        DiscoverMessage::SeriesPosterAction(index, Message::ImageLoaded(image))
                    },
                )
            } else {
                Command::none()
            };

            (poster, series_image_command)
        }

        pub fn update(&mut self, message: Message) -> Command<DiscoverMessage> {
            match message {
                Message::ImageLoaded(image) => self.image = image,
                Message::SeriesPosterPressed(series_id) => {
                    return Command::perform(async {}, move |_| {
                        DiscoverMessage::SeriesSelected(series_id)
                    })
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).height(120);
                content = content.push(image);
            };

            content = content.push(text(&self.series_information.name).size(15));

            // content.push(text(&self.episode.name)).into()
            mouse_area(content)
                .on_press(Message::SeriesPosterPressed(self.series_information.id))
                .into()
        }
    }
}
