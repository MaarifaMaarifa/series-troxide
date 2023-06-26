use crate::core::api::episodes_information::Episode;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::tv_schedule::get_episodes_with_date;
use crate::core::api::updates::show_updates::*;
use crate::gui::Message as GuiMessage;
use episode_poster::Message as EpisodePosterMessage;
use searching::Message as SearchMessage;
use series_updates_poster::Message as SeriesPosterMessage;

use iced::{
    widget::{column, container, row, scrollable, text},
    Command, Element, Length, Renderer,
};

use iced_aw::floating_element;
use iced_aw::Spinner;
use iced_aw::{floating_element::Offset, wrap::Wrap};

#[derive(Default, PartialEq)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Clone, Debug)]
pub enum Message {
    LoadSchedule,
    ScheduleLoaded(Vec<Episode>),
    SeriesUpdatesLoaded(Vec<SeriesMainInformation>),
    EpisodePosterAction(
        /*episode poster index*/ usize,
        Box<EpisodePosterMessage>,
    ),
    SeriesPosterAction(/*series poster index*/ usize, Box<SeriesPosterMessage>),
    SearchAction(SearchMessage),
    SeriesSelected(/*series_id*/ Box<SeriesMainInformation>),
    SeriesResultSelected(/*series_id*/ u32),
    ShowOverlay,
    HideOverlay,
}

#[derive(Default)]
pub struct Discover {
    load_state: LoadState,
    show_overlay: bool,
    search_state: searching::Search,
    new_episodes: Vec<episode_poster::EpisodePoster>,
    series_updates: Vec<series_updates_poster::SeriesPoster>,
}

impl Discover {
    pub fn new() -> (Self, Command<GuiMessage>) {
        let series_updates_command = Command::perform(
            get_show_updates(UpdateTimestamp::Day, Some(100)),
            |series| {
                GuiMessage::DiscoverAction(Message::SeriesUpdatesLoaded(
                    series.expect("Failed to load series updates"),
                ))
            },
        );

        let new_episodes_command = Command::perform(get_episodes_with_date(None), |episodes| {
            GuiMessage::DiscoverAction(Message::ScheduleLoaded(
                episodes.expect("Failed to load episodes schedule"),
            ))
        });

        (
            Self::default(),
            Command::batch([series_updates_command, new_episodes_command]),
        )
    }
    pub fn update(&mut self, message: Message) -> Command<GuiMessage> {
        if let searching::LoadState::Loaded = self.search_state.load_state {
            self.show_overlay = true;
        }
        match message {
            Message::LoadSchedule => {
                self.load_state = LoadState::Loading;

                let series_updates_command = Command::perform(
                    get_show_updates(UpdateTimestamp::Day, Some(100)),
                    |series| {
                        GuiMessage::DiscoverAction(Message::SeriesUpdatesLoaded(
                            series.expect("Failed to load series updates"),
                        ))
                    },
                );

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
                Command::batch(commands).map(GuiMessage::DiscoverAction)
            }
            Message::EpisodePosterAction(index, message) => self.new_episodes[index]
                .update(*message)
                .map(GuiMessage::DiscoverAction),
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
            Message::SeriesPosterAction(index, message) => self.series_updates[index]
                .update(*message)
                .map(GuiMessage::DiscoverAction),
            Message::SearchAction(message) => {
                if let SearchMessage::SeriesResultPressed(series_id) = message {
                    return Command::perform(async {}, move |_| {
                        GuiMessage::DiscoverAction(Message::SeriesResultSelected(series_id))
                    });
                };
                self.search_state
                    .update(message)
                    .map(GuiMessage::DiscoverAction)
            }
            Message::ShowOverlay => {
                self.show_overlay = true;
                Command::none()
            }
            Message::HideOverlay => {
                self.show_overlay = false;
                Command::none()
            }
            Message::SeriesSelected(_) => {
                unreachable!("Discover View should not handle Series View")
            }
            Message::SeriesResultSelected(_) => {
                unreachable!("Discover View should not handle Series View")
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let underlay: Element<'_, Message, Renderer> = match self.load_state {
            LoadState::Loading => container(Spinner::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            LoadState::Loaded => column!(scrollable(
                column!(load_new_episodes(self), load_series_updates(self)).spacing(20)
            )
            .width(Length::Fill))
            .into(),
        };

        let offset_distance = Offset { x: -140.0, y: 0.0 };

        let content = floating_element::FloatingElement::new(underlay, || {
            self.search_state.view().1.map(Message::SearchAction)
        })
        .anchor(floating_element::Anchor::North)
        .offset(offset_distance)
        .hide(!self.show_overlay);

        column![
            self.search_state.view().0.map(Message::SearchAction),
            content
        ]
        .spacing(2)
        .into()
    }
}

fn load_new_episodes(discover_view: &Discover) -> Element<'_, Message, Renderer> {
    let title = text("New Episodes Airing Today").size(25);
    let new_episode = Wrap::with_elements(
        discover_view
            .new_episodes
            .iter()
            .enumerate()
            .map(|(index, poster)| {
                poster
                    .view()
                    .map(move |m| Message::EpisodePosterAction(index, Box::new(m)))
            })
            .collect(),
    )
    .spacing(5.0)
    .padding(5.0);
    column!(title, new_episode).into()
}

fn load_series_updates(discover_view: &Discover) -> Element<'_, Message, Renderer> {
    let title = text("Trending Shows").size(25);

    let trending_shows = Wrap::with_elements(
        discover_view
            .series_updates
            .iter()
            .enumerate()
            .map(|(index, poster)| {
                poster
                    .view()
                    .map(move |m| Message::SeriesPosterAction(index, Box::new(m)))
            })
            .collect(),
    )
    .spacing(5.0)
    .padding(5.0);
    column!(title, trending_shows).into()
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
        ImageLoaded(Box<Option<Vec<u8>>>),
        SeriesInformationLoaded(Box<SeriesMainInformation>),
        EpisodePosterPressed(/*series_id*/ Box<SeriesMainInformation>),
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
                        Box::new(Message::SeriesInformationLoaded(Box::new(series))),
                    )
                },
            );

            (poster, series_information_command)
        }

        pub fn update(&mut self, message: Message) -> Command<DiscoverMessage> {
            match message {
                Message::ImageLoaded(image) => self.image = *image,
                Message::SeriesInformationLoaded(series_info) => {
                    let series_image_url = series_info.image.clone();
                    let poster_index = self.index;
                    self.series_belonging = Some(*series_info);

                    if let Some(image) = series_image_url {
                        return Command::perform(
                            load_image(image.medium_image_url),
                            move |image| {
                                DiscoverMessage::EpisodePosterAction(
                                    poster_index,
                                    Box::new(Message::ImageLoaded(Box::new(image))),
                                )
                            },
                        );
                    }
                }
                Message::EpisodePosterPressed(series_information) => {
                    return Command::perform(async {}, move |_| {
                        DiscoverMessage::SeriesSelected(series_information)
                    })
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            };

            if let Some(series_info) = &self.series_belonging {
                content = content.push(
                    text(&series_info.name)
                        .size(15)
                        .width(100)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                )
            }

            // content.push(text(&self.episode.name)).into()
            if let Some(series_info) = &self.series_belonging {
                mouse_area(content)
                    .on_press(Message::EpisodePosterPressed(Box::new(series_info.clone())))
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
        SeriesPosterPressed(/*series_id*/ Box<SeriesMainInformation>),
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
                        DiscoverMessage::SeriesPosterAction(
                            index,
                            Box::new(Message::ImageLoaded(image)),
                        )
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
                Message::SeriesPosterPressed(series_information) => {
                    return Command::perform(async {}, move |_| {
                        DiscoverMessage::SeriesSelected(series_information)
                    })
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            };

            content = content.push(
                text(&self.series_information.name)
                    .size(15)
                    .width(100)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            );

            // content.push(text(&self.episode.name)).into()
            mouse_area(content)
                .on_press(Message::SeriesPosterPressed(Box::new(
                    self.series_information.clone(),
                )))
                .into()
        }
    }
}

mod searching {

    use iced::theme;
    use iced::widget::{
        column, container, horizontal_space, image, mouse_area, row, text, text_input,
        vertical_space, Column,
    };
    use iced::{Command, Element, Length, Renderer};
    use tokio::task::JoinHandle;

    use super::Message as DiscoverMessage;
    use crate::core::api::load_image;
    use crate::core::api::series_searching;

    #[derive(Default)]
    pub enum LoadState {
        Loaded,
        Loading,
        #[default]
        NotLoaded,
    }

    #[derive(Clone, Debug)]
    pub enum Message {
        SearchTermChanged(String),
        SearchTermSearched,
        SearchSuccess(Vec<series_searching::SeriesSearchResult>),
        SearchFail,
        ImagesLoaded(Vec<Option<Vec<u8>>>),
        SeriesResultPressed(/*series id*/ u32),
    }

    #[derive(Default)]
    pub struct Search {
        search_term: String,
        series_search_result: Vec<series_searching::SeriesSearchResult>,
        series_search_results_images: Vec<Option<Vec<u8>>>,
        pub load_state: LoadState,
    }

    impl Search {
        pub fn update(&mut self, message: Message) -> Command<DiscoverMessage> {
            match message {
                Message::SearchTermChanged(term) => {
                    self.search_term = term;
                    return Command::none();
                }
                Message::SearchTermSearched => {
                    self.load_state = LoadState::Loading;

                    let series_result = series_searching::search_series(self.search_term.clone());

                    return Command::perform(series_result, |res| match res {
                        Ok(res) => DiscoverMessage::SearchAction(Message::SearchSuccess(res)),
                        Err(err) => {
                            println!("{:?}", err);
                            DiscoverMessage::SearchAction(Message::SearchFail)
                        }
                    });
                }
                Message::SearchSuccess(res) => {
                    self.load_state = LoadState::Loaded;
                    self.series_search_results_images.clear();
                    self.series_search_result = res.clone();
                    return Command::perform(load_series_result_images(res), |images| {
                        DiscoverMessage::SearchAction(Message::ImagesLoaded(images))
                    });
                }
                Message::SearchFail => panic!("Series Search Failed"),
                Message::ImagesLoaded(images) => self.series_search_results_images = images,
                Message::SeriesResultPressed(_) => {
                    unreachable!("Search page should not handle series page result")
                }
            }
            Command::none()
        }

        pub fn view(
            &self,
        ) -> (
            Element<'_, Message, Renderer>,
            Element<'_, Message, Renderer>,
        ) {
            let search_bar = column!(
                vertical_space(10),
                text_input("Search Series", &self.search_term)
                    .width(300)
                    .on_input(Message::SearchTermChanged)
                    .on_submit(Message::SearchTermSearched)
            )
            .width(Length::Fill)
            .align_items(iced::Alignment::Center);

            let menu_widgets = match self.load_state {
                LoadState::Loaded => load(
                    &self.series_search_result,
                    &self.series_search_results_images,
                ),
                _ => vec![],
            };

            let menu_widgets = container(
                Column::with_children(menu_widgets)
                    .width(Length::Fill)
                    .padding(20)
                    .spacing(5),
            )
            .style(theme::Container::Custom(Box::new(ContainerTheme)
                as Box<dyn container::StyleSheet<Style = iced::theme::Theme>>));

            (search_bar.into(), menu_widgets.into())
        }
    }

    fn load<'a>(
        series_result: &'a [series_searching::SeriesSearchResult],
        series_images: &Vec<Option<Vec<u8>>>,
    ) -> Vec<Element<'a, Message, Renderer>> {
        let mut results = Vec::new();

        for (index, series_result) in series_result.iter().enumerate() {
            results.push(series_result_widget(
                series_result,
                if series_images.is_empty() {
                    None
                } else {
                    series_images[index].clone().take()
                },
            ));
        }
        results
    }

    pub fn series_result_widget(
        series_result: &series_searching::SeriesSearchResult,
        image_bytes: Option<Vec<u8>>,
    ) -> iced::Element<'_, Message, Renderer> {
        let mut row = row!();

        if let Some(image_bytes) = image_bytes {
            let image_handle = image::Handle::from_memory(image_bytes);

            let image = image(image_handle).height(60);
            row = row
                .push(horizontal_space(5))
                .push(image)
                .push(horizontal_space(5));
        }

        // Getting the series genres
        let genres = if !series_result.show.genres.is_empty() {
            let mut genres = String::from("Genres: ");

            let mut series_result_iter = series_result.show.genres.iter().peekable();
            while let Some(genre) = series_result_iter.next() {
                genres.push_str(genre);
                if series_result_iter.peek().is_some() {
                    genres.push_str(", ");
                }
            }
            genres
        } else {
            String::new()
        };

        let mut column = column!(
            text(&series_result.show.name).size(20),
            text(genres).size(15),
        );

        if let Some(premier) = &series_result.show.premiered {
            column = column.push(text(format!("Premiered: {}", premier)).size(13));
        }

        mouse_area(row.push(column))
            .on_press(Message::SeriesResultPressed(series_result.show.id))
            .into()
    }

    async fn load_series_result_images(
        series_results: Vec<series_searching::SeriesSearchResult>,
    ) -> Vec<Option<Vec<u8>>> {
        let mut loaded_results = Vec::with_capacity(series_results.len());
        let handles: Vec<JoinHandle<Option<Vec<u8>>>> = series_results
            .into_iter()
            .map(|result| {
                tokio::task::spawn(async {
                    if let Some(url) = result.show.image {
                        load_image(url.medium_image_url).await
                    } else {
                        None
                    }
                })
            })
            .collect();

        for handle in handles {
            let loaded_result = handle
                .await
                .expect("Failed to await all the search images handles");
            loaded_results.push(loaded_result)
        }
        loaded_results
    }

    pub struct ContainerTheme;

    impl iced::widget::container::StyleSheet for ContainerTheme {
        type Style = iced::Theme;

        fn appearance(
            &self,
            style: &<Self as container::StyleSheet>::Style,
        ) -> container::Appearance {
            let background_color = match style {
                iced::Theme::Light => theme::palette::Palette::LIGHT.background,
                iced::Theme::Dark => theme::palette::Palette::DARK.background,
                iced::Theme::Custom(_) => todo!(),
            };
            container::Appearance {
                background: background_color.into(),
                ..Default::default()
            }
        }
    }
}
