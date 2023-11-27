use std::rc::Rc;

use iced::widget::{column, container, text, Column};
use iced::{Alignment, Command, Element, Length};
use iced_aw::Spinner;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::{
    api::tv_maze::episodes_information::EpisodeReleaseTime, caching::episode_list::EpisodeList,
};
use crate::gui::message::IndexedMessage;
use crate::gui::styles;
use season::{Message as SeasonMessage, Season};

#[derive(Debug, Clone)]
pub enum Message {
    Season(IndexedMessage<usize, SeasonMessage>),
    EpisodeListLoaded(EpisodeList),
}

pub struct Seasons {
    series_name: String,
    series_id: u32,
    episode_list: Option<Rc<EpisodeList>>,
    seasons: Vec<Season>,
}

impl Seasons {
    pub fn new(series_id: u32, series_name: String) -> (Self, Command<Message>) {
        (
            Self {
                series_name,
                series_id,
                episode_list: None,
                seasons: vec![],
            },
            Command::perform(
                async move {
                    EpisodeList::new(series_id)
                        .await
                        .expect("failed to get episodes list")
                },
                Message::EpisodeListLoaded,
            ),
        )
    }

    pub fn get_next_episode_and_release_time(&self) -> Option<(&Episode, EpisodeReleaseTime)> {
        self.episode_list.as_ref().and_then(|episode_list| {
            episode_list
                .get_next_episode_to_air_and_time()
                .map(|(episode, release_time)| (episode, release_time))
        })
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Season(message) => self.seasons[message.index()]
                .update(message)
                .map(Message::Season),
            Message::EpisodeListLoaded(episode_list) => {
                let season_numbers = episode_list.get_season_numbers();

                self.episode_list = Some(Rc::new(episode_list));

                self.seasons = season_numbers
                    .into_iter()
                    .enumerate()
                    .map(|(index, season)| {
                        Season::new(
                            index,
                            self.series_id,
                            self.episode_list
                                .clone()
                                .unwrap_or_else(|| unreachable!("EpisodeList should be present")),
                            self.series_name.to_string(),
                            season,
                        )
                    })
                    .collect();

                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let seasons_body = column![text("Seasons").size(21)]
            .align_items(Alignment::Center)
            .spacing(10);

        let content = if self.episode_list.is_none() {
            container(seasons_body.push(Spinner::new()))
                .width(700)
                .center_x()
        } else if self.seasons.is_empty() {
            container(seasons_body.push(text("No seasons found")))
                .width(700)
                .center_x()
        } else {
            container(
                seasons_body.push(
                    Column::with_children(
                        self.seasons
                            .iter()
                            .map(|season| season.view().map(Message::Season))
                            .collect(),
                    )
                    .padding(5)
                    .spacing(5)
                    .align_items(Alignment::Center),
                ),
            )
        }
        .padding(10)
        .style(styles::container_styles::first_class_container_rounded_theme());

        container(content)
            .width(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}

mod season {
    use std::rc::Rc;

    use iced::widget::{button, checkbox, column, container, progress_bar, row, svg, text, Column};
    use iced::{Command, Element, Length, Renderer};
    use iced_aw::Spinner;

    use crate::core::api::tv_maze::episodes_information::Episode as EpisodeInfo;
    use crate::core::caching::episode_list::{EpisodeList, TotalEpisodes};
    use crate::core::database;
    use crate::core::database::AddResult;
    use crate::gui::assets::icons::{CHEVRON_DOWN, CHEVRON_UP};
    use crate::gui::message::IndexedMessage;
    use crate::gui::styles;
    use crate::gui::troxide_widget::episode_widget::{
        Episode, Message as EpisodeMessage, PosterType,
    };

    #[derive(Clone, Debug)]
    pub enum Message {
        CheckboxPressed,
        TrackCommandComplete(AddResult),
        Expand,
        Episode(IndexedMessage<usize, EpisodeMessage>),
    }

    #[derive(Clone)]
    pub struct Season {
        index: usize,
        series_id: u32,
        episode_list: Rc<EpisodeList>,
        series_name: String,
        season_number: u32,
        total_episodes: TotalEpisodes,
        episodes: Vec<Episode>,
        is_expanded: bool,
    }

    impl Season {
        pub fn new(
            index: usize,
            series_id: u32,
            episode_list: Rc<EpisodeList>,
            series_name: String,
            season_number: u32,
        ) -> Self {
            let total_episodes = episode_list.get_season_total_episodes(season_number);
            Self {
                index,
                series_id,
                episode_list,
                series_name,
                season_number,
                total_episodes,
                episodes: vec![],
                is_expanded: false,
            }
        }
        pub fn update(
            &mut self,
            message: IndexedMessage<usize, Message>,
        ) -> Command<IndexedMessage<usize, Message>> {
            match message.message() {
                Message::CheckboxPressed => {
                    let series_id = self.series_id;
                    let series_name = self.series_name.clone();
                    let season_number = self.season_number;
                    let total_episodes = self.total_episodes.get_all_episodes();
                    let index = self.index;

                    return Command::perform(
                        async move {
                            if let Some(mut series) = database::DB.get_series(series_id) {
                                series
                                    .add_episodes(season_number, 1..=total_episodes as u32)
                                    .await
                            } else {
                                let mut series = database::Series::new(series_name, series_id);
                                series
                                    .add_episodes(season_number, 1..=total_episodes as u32)
                                    .await
                            }
                        },
                        Message::TrackCommandComplete,
                    )
                    .map(move |message| IndexedMessage::new(index, message));
                }
                Message::Expand => {
                    self.is_expanded = !self.is_expanded;

                    // preventing reloading episodes when already loaded
                    // when expanding and shrinking the season widget multiple times
                    if !self.episodes.is_empty() {
                        return Command::none();
                    }

                    let episode_infos: Vec<EpisodeInfo> = self
                        .episode_list
                        .get_episodes(self.season_number)
                        .into_iter()
                        .cloned()
                        .collect();

                    let epis: Vec<(Episode, Command<IndexedMessage<usize, EpisodeMessage>>)> =
                        episode_infos
                            .into_iter()
                            .enumerate()
                            .map(|(index, info)| {
                                Episode::new(index, self.series_id, self.series_name.clone(), info)
                            })
                            .collect();

                    let index = self.index;
                    let mut commands = Vec::with_capacity(epis.len());
                    let mut episodes = Vec::with_capacity(epis.len());
                    for (episode, command) in epis {
                        episodes.push(episode);
                        commands.push(command);
                    }

                    self.episodes = episodes;
                    return Command::batch(commands)
                        .map(Message::Episode)
                        .map(move |message| IndexedMessage::new(index, message));
                }
                Message::Episode(message) => {
                    let season_index = self.index;
                    return self.episodes[message.index()]
                        .update(message)
                        .map(Message::Episode)
                        .map(move |message| IndexedMessage::new(season_index, message));
                }
                Message::TrackCommandComplete(add_result) => {
                    if let AddResult::None = add_result {
                        if let Some(mut series) = database::DB.get_series(self.series_id) {
                            series.remove_season(self.season_number);
                        }
                    }
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, IndexedMessage<usize, Message>, Renderer> {
            let tracked_episodes = database::DB
                .get_series(self.series_id)
                .map(|series| {
                    series
                        .get_season(self.season_number)
                        .map(|season| season.get_total_episodes())
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            let track_checkbox = checkbox(
                "",
                (self.total_episodes.get_all_watchable_episodes() == tracked_episodes)
                    && (tracked_episodes != 0),
                |_| Message::CheckboxPressed,
            );
            let season_name = text(format!("Season {}", self.season_number)).width(80);

            let season_progress = progress_bar(
                0.0..=self.total_episodes.get_all_episodes() as f32,
                tracked_episodes as f32,
            )
            .height(10)
            .width(500);

            let episodes_progress = text(format!(
                "{}/{}",
                tracked_episodes,
                self.total_episodes.get_all_episodes()
            ))
            .width(50);

            let expand_button = if self.is_expanded {
                let svg_handle = svg::Handle::from_memory(CHEVRON_UP);
                let up_icon = svg(svg_handle)
                    .width(Length::Shrink)
                    .style(styles::svg_styles::colored_svg_theme());
                button(up_icon)
                    .on_press(Message::Expand)
                    .style(styles::button_styles::transparent_button_theme())
            } else {
                let svg_handle = svg::Handle::from_memory(CHEVRON_DOWN);
                let down_icon = svg(svg_handle)
                    .width(Length::Shrink)
                    .style(styles::svg_styles::colored_svg_theme());
                button(down_icon)
                    .on_press(Message::Expand)
                    .style(styles::button_styles::transparent_button_theme())
            };

            let content = row![
                track_checkbox,
                season_name,
                season_progress,
                episodes_progress,
                expand_button,
            ]
            .spacing(5);

            let mut content = column!(content);
            if self.is_expanded {
                if self.episodes.is_empty() {
                    content = content.push(container(Spinner::new()))
                } else {
                    content = content.push(
                        Column::with_children(
                            self.episodes
                                .iter()
                                .map(|episode| {
                                    episode.view(PosterType::Season).map(Message::Episode)
                                })
                                .collect(),
                        )
                        .spacing(3),
                    );
                }
            }

            let element: Element<'_, Message, Renderer> = content.into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }
    }
}
