use iced::widget::{button, checkbox, column, progress_bar, row, text, Column};
use iced::{Command, Element, Renderer};
use tokio::task::JoinHandle;

use self::episode_widget::Episode;

use super::Message as SeriesMessage;
use crate::core::api::episodes_information::{get_episode_information, Episode as EpisodeInfo};
use crate::core::api::seasons_list::Season as SeasonInfo;
use episode_widget::Message as EpisodeMessage;

#[derive(Clone, Debug)]
pub enum Message {
    CheckboxPressed(bool),
    Expand,
    EpisodesLoaded(Vec<EpisodeInfo>),
    EpisodeAction(usize, EpisodeMessage),
}

#[derive(Clone)]
pub struct Season {
    index: usize,
    series_id: u32,
    season: SeasonInfo,
    episodes: Vec<episode_widget::Episode>,
    is_tracked: bool,
    is_expanded: bool,
}

impl Season {
    pub fn new(index: usize, season_info: SeasonInfo, series_id: u32) -> Self {
        Self {
            index,
            series_id,
            season: season_info,
            episodes: vec![],
            is_tracked: false,
            is_expanded: false,
        }
    }
    pub fn update(&mut self, message: Message) -> Command<SeriesMessage> {
        match message {
            Message::CheckboxPressed(tracking_status) => {
                if self.season.episode_order.is_some() {
                    self.is_tracked = tracking_status;
                }
            }
            Message::Expand => {
                self.is_expanded = !self.is_expanded;

                // preventing reloading episodes when already loaded
                if !self.episodes.is_empty() {
                    return Command::none();
                }

                if let Some(total_episodes) = self.season.episode_order {
                    let series_id = self.series_id;
                    let season_number = self.season.number;
                    let series_index = self.index;
                    return Command::perform(
                        async move {
                            load_episode_infos(total_episodes, series_id, season_number).await
                        },
                        move |episode_infos| {
                            SeriesMessage::SeasonAction(
                                series_index,
                                Box::new(Message::EpisodesLoaded(episode_infos)),
                            )
                        },
                    );
                }
            }
            Message::EpisodesLoaded(episode_infos) => {
                let epis: Vec<(Episode, Command<Message>)> = episode_infos
                    .into_iter()
                    .enumerate()
                    .map(|(index, info)| episode_widget::Episode::new(index, info))
                    .collect();

                let mut commands = Vec::with_capacity(epis.len());
                let mut episodes = Vec::with_capacity(epis.len());
                for (episode, command) in epis {
                    episodes.push(episode);
                    let index = self.index;
                    commands.push(
                        command.map(move |m| SeriesMessage::SeasonAction(index, Box::new(m))),
                    );
                }

                self.episodes = episodes;
                return Command::batch(commands);
            }
            Message::EpisodeAction(index, message) => {
                let season_index = self.index;
                return self.episodes[index]
                    .update(message)
                    .map(move |m| SeriesMessage::SeasonAction(season_index, Box::new(m)));
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let track_checkbox = checkbox("", self.is_tracked, |tracking| {
            Message::CheckboxPressed(tracking)
        });
        let season_name = text(format!("Season {}", self.season.number));
        let season_progress = if let Some(episodes_number) = self.season.episode_order {
            progress_bar(0.0..=episodes_number as f32, 0.0)
                .height(10)
                .width(500)
        } else {
            progress_bar(0.0..=0.0, 0.0).height(10).width(500)
        };
        let episodes_progress = text(format!("{}/{}", 0, self.season.episode_order.unwrap_or(0)));
        let expand_button = button(">").on_press(Message::Expand);
        let content = row!(
            track_checkbox,
            season_name,
            season_progress,
            episodes_progress,
            expand_button,
        );

        let mut content = column!(content);
        if self.is_expanded && !self.episodes.is_empty() {
            content = content.push(Column::with_children(
                self.episodes
                    .iter()
                    .enumerate()
                    .map(|(index, episode)| {
                        episode
                            .view()
                            .map(move |m| Message::EpisodeAction(index, m))
                    })
                    .collect(),
            ));
        }

        content.into()
    }
}

async fn load_episode_infos(
    total_episode: u32,
    series_id: u32,
    season_number: u32,
) -> Vec<EpisodeInfo> {
    let mut loaded_results = Vec::with_capacity(total_episode as usize);
    let handles: Vec<JoinHandle<EpisodeInfo>> = (1..=total_episode)
        .map(|episode_number| {
            tokio::task::spawn(async move {
                get_episode_information(series_id, season_number, episode_number)
                    .await
                    .expect("could not get all the episode information")
            })
        })
        .collect();

    for handle in handles {
        let loaded_result = handle
            .await
            .expect("Failed to await all the episode infos handles");
        loaded_results.push(loaded_result)
    }
    loaded_results
}

mod episode_widget {
    use super::Message as SeasonMessage;
    use crate::core::api::{episodes_information::Episode as EpisodeInfo, load_image};
    use iced::{
        widget::{checkbox, column, horizontal_space, image, row, text, Row, Text},
        Command, Element, Length, Renderer,
    };

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(Option<Vec<u8>>),
        TrackCheckboxPressed(bool),
    }

    #[derive(Clone)]
    pub struct Episode {
        // index: usize,
        episode_information: EpisodeInfo,
        episode_image: Option<Vec<u8>>,
        is_tracked: bool,
    }

    impl Episode {
        pub fn new(
            index: usize,
            episode_information: EpisodeInfo,
        ) -> (Self, Command<SeasonMessage>) {
            let episode_image = episode_information.image.clone();
            let episode = Self {
                episode_information,
                episode_image: None,
                is_tracked: false,
            };

            let command = if let Some(image) = episode_image {
                Command::perform(load_image(image.medium_image_url), move |image| {
                    SeasonMessage::EpisodeAction(index, Message::ImageLoaded(image))
                })
            } else {
                Command::none()
            };

            (episode, command)
        }

        pub fn update(&mut self, message: Message) -> Command<SeasonMessage> {
            match message {
                Message::ImageLoaded(image) => self.episode_image = image,
                Message::TrackCheckboxPressed(val) => self.is_tracked = val,
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = row!();
            if let Some(image_bytes) = self.episode_image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).height(60);
                content = content.push(image);
            };
            let info = column!(
                heading_widget(&self.episode_information, self.is_tracked),
                airdate_widget(&self.episode_information),
                airstamp_widget(&self.episode_information),
                summary_widget(&self.episode_information)
            )
            .padding(5);
            content.push(info).padding(5).width(600).into()
        }
    }

    fn summary_widget(episode_information: &EpisodeInfo) -> Text<'static, Renderer> {
        if let Some(summary) = &episode_information.summary {
            text(summary).size(15)
        } else {
            text("")
        }
    }

    fn airdate_widget(episode_information: &EpisodeInfo) -> Text<'static, Renderer> {
        if let Some(airdate) = &episode_information.airdate {
            text(airdate).size(15)
        } else {
            text("")
        }
    }

    fn airstamp_widget(episode_information: &EpisodeInfo) -> Text<'static, Renderer> {
        text(&episode_information.airstamp).size(15)
    }

    fn heading_widget(
        episode_information: &EpisodeInfo,
        track_status: bool,
    ) -> Row<'static, Message, Renderer> {
        let tracking_checkbox = checkbox("", track_status, Message::TrackCheckboxPressed);
        row!(
            text(format!(
                "S{}E{}",
                parse_season_episode_number(episode_information.season),
                parse_season_episode_number(episode_information.number)
            ),),
            text(&episode_information.name).size(17),
            horizontal_space(Length::Fill),
            tracking_checkbox.size(17),
        )
        .spacing(5)
    }

    fn parse_season_episode_number(number: u32) -> String {
        if number < 10_u32 {
            format!("0{}", number)
        } else {
            number.to_string()
        }
    }
}
