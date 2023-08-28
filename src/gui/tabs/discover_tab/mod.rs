use std::sync::mpsc;

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::caching::tv_schedule::{get_series_with_country, get_series_with_date};
use crate::core::settings_config::locale_settings;
use crate::gui::assets::icons::BINOCULARS_FILL;
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use full_schedule_posters::{FullSchedulePosters, Message as FullSchedulePostersMessage};
use searching::Message as SearchMessage;

use iced::widget::{column, container, scrollable, text, vertical_space};
use iced::{Command, Element, Length, Renderer};

use iced_aw::floating_element;
use iced_aw::wrap::Wrap;
use iced_aw::Spinner;

mod searching;

#[derive(Default, PartialEq)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Default)]
struct LoadStatus {
    global_series: LoadState,
    local_series: LoadState,
}

#[derive(Clone, Debug)]
pub enum Message {
    Reload,
    GlobalSeriesLoaded(Vec<SeriesMainInformation>),
    LocalSeriesLoaded(Vec<SeriesMainInformation>),
    GlobalSeries(SeriesPosterMessage),
    LocalSeries(SeriesPosterMessage),
    FullSchedulePosters(FullSchedulePostersMessage),
    Search(SearchMessage),
    SeriesSelected(Box<SeriesMainInformation>),
    ShowSearchResults,
    HideSearchResults,
    EscapeKeyPressed,
}

pub struct DiscoverTab {
    load_status: LoadStatus,
    show_search_results: bool,
    search_state: searching::Search,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    country_name: String,

    new_global_series: Vec<SeriesPoster>,
    new_local_series: Vec<SeriesPoster>,
    full_schedule_series: FullSchedulePosters,
}

impl DiscoverTab {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        let (full_schedule_series, full_schedule_command) =
            FullSchedulePosters::new(series_page_sender.clone());

        (
            Self {
                load_status: LoadStatus::default(),
                show_search_results: false,
                search_state: searching::Search::default(),
                new_global_series: vec![],
                new_local_series: vec![],
                full_schedule_series,
                series_page_sender,
                country_name: locale_settings::get_country_name_from_settings(),
            },
            Command::batch([
                load_discover_schedule_command(),
                full_schedule_command.map(Message::FullSchedulePosters),
            ]),
        )
    }

    pub fn refresh(&mut self) -> Command<Message> {
        let current_country_name = locale_settings::get_country_name_from_settings();
        if self.country_name != current_country_name {
            self.load_status.local_series = LoadState::Loading;
            self.country_name = current_country_name;
            load_local_aired_series()
        } else {
            Command::none()
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::subscription::events_with(|event, _| {
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) = event
            {
                if key_code == iced::keyboard::KeyCode::Escape && modifiers.is_empty() {
                    return Some(Message::EscapeKeyPressed);
                }
                if key_code == iced::keyboard::KeyCode::F5 && modifiers.is_empty() {
                    return Some(Message::Reload);
                }
            }
            None
        })
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Reload => {
                let mut load_commands = [Command::none(), Command::none()];

                if let LoadState::Loaded = &self.load_status.local_series {
                    self.load_status.local_series = LoadState::Loading;
                    load_commands[0] = load_local_aired_series();
                }
                if let LoadState::Loaded = &self.load_status.global_series {
                    self.load_status.global_series = LoadState::Loading;
                    load_commands[1] = load_global_aired_series();
                }

                Command::batch(load_commands)
            }
            Message::GlobalSeriesLoaded(series_infos) => {
                self.load_status.global_series = LoadState::Loaded;

                let mut series_posters = Vec::with_capacity(series_infos.len());
                let mut commands = Vec::with_capacity(series_infos.len());
                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (poster, command) = SeriesPoster::new(index, series_info);
                    series_posters.push(poster);
                    commands.push(command);
                }

                self.new_global_series = series_posters;
                Command::batch(commands).map(Message::GlobalSeries)
            }
            Message::GlobalSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.new_global_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::GlobalSeries)
            }
            Message::Search(message) => {
                if let SearchMessage::SeriesResultPressed(series_info) = message {
                    self.series_page_sender
                        .send(*series_info)
                        .expect("failed to send series page");
                    self.show_search_results = false;
                    return Command::none();
                };
                self.search_state.update(message)
            }
            Message::ShowSearchResults => {
                self.show_search_results = true;
                Command::none()
            }
            Message::HideSearchResults => {
                self.show_search_results = false;
                Command::none()
            }
            Message::SeriesSelected(series_info) => {
                self.series_page_sender
                    .send(*series_info)
                    .expect("failed to send series page");
                Command::none()
            }
            Message::LocalSeriesLoaded(series_infos) => {
                self.load_status.local_series = LoadState::Loaded;

                let mut series_posters = Vec::with_capacity(series_infos.len());
                let mut commands = Vec::with_capacity(series_infos.len());
                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (poster, command) = SeriesPoster::new(index, series_info);
                    series_posters.push(poster);
                    commands.push(command);
                }
                self.new_local_series = series_posters;
                Command::batch(commands).map(Message::LocalSeries)
            }
            Message::LocalSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.new_local_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::LocalSeries)
            }
            Message::EscapeKeyPressed => {
                self.show_search_results = false;
                Command::none()
            }
            Message::FullSchedulePosters(message) => self
                .full_schedule_series
                .update(message)
                .map(Message::FullSchedulePosters),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let underlay: Element<'_, Message, Renderer> = scrollable(
            column!(
                series_posters_loader(
                    "Shows Airing Today Globally",
                    &self.load_status.global_series,
                    &self.new_global_series
                )
                .map(Message::GlobalSeries),
                series_posters_loader(
                    &format!("Shows Airing Today in {}", self.country_name),
                    &self.load_status.local_series,
                    &self.new_local_series
                )
                .map(Message::LocalSeries),
                self.full_schedule_series
                    .view()
                    .map(Message::FullSchedulePosters)
            )
            .spacing(20),
        )
        .width(Length::Fill)
        .into();

        let content = floating_element::FloatingElement::new(
            underlay,
            self.search_state.view().1.map(Message::Search),
        )
        .anchor(floating_element::Anchor::North)
        .hide(!self.show_search_results);

        column![self.search_state.view().0.map(Message::Search), content]
            .spacing(2)
            .padding(10)
            .into()
    }
}

impl DiscoverTab {
    pub fn title() -> String {
        "Discover".to_owned()
    }

    pub fn tab_label() -> super::TabLabel {
        super::TabLabel::new(Self::title(), BINOCULARS_FILL)
    }
}

/// Loads the locally aired series picking up the country set from the settings
fn load_local_aired_series() -> Command<Message> {
    Command::perform(
        async {
            let country_code = locale_settings::get_country_code_from_settings();
            get_series_with_country(&country_code).await
        },
        |series| Message::LocalSeriesLoaded(series.expect("failed to load series schedule")),
    )
}

/// Loads the globally aired series
fn load_global_aired_series() -> Command<Message> {
    Command::perform(get_series_with_date(None), |series| {
        Message::GlobalSeriesLoaded(series.expect("failed to load series schedule"))
    })
}

/// Loads series updates, globally and locally aired series all at once
fn load_discover_schedule_command() -> Command<Message> {
    Command::batch([load_global_aired_series(), load_local_aired_series()])
}

/// wraps the given series posters and places a title above them
fn series_posters_loader<'a>(
    title: &str,
    load_state: &LoadState,
    posters: &'a [SeriesPoster],
) -> Element<'a, SeriesPosterMessage, Renderer> {
    let title = text(title).size(21);

    if let LoadState::Loading = load_state {
        let spinner = container(Spinner::new())
            .center_x()
            .center_y()
            .height(100)
            .width(Length::Fill);

        return column!(title, vertical_space(10), spinner)
            .width(Length::Fill)
            .padding(10)
            .into();
    }

    if posters.is_empty() {
        let text = container(text("No Series Found"))
            .center_x()
            .center_y()
            .height(100)
            .width(Length::Fill);
        column!(title, vertical_space(10), text)
            .width(Length::Fill)
            .padding(10)
            .into()
    } else {
        let wrapped_posters =
            Wrap::with_elements(posters.iter().map(|poster| poster.normal_view()).collect())
                .spacing(5.0)
                .line_spacing(5.0)
                .padding(5.0);

        column!(title, wrapped_posters)
            .spacing(5)
            .width(Length::Fill)
            .padding(10)
            .into()
    }
}

mod full_schedule_posters {
    use std::collections::HashMap;
    use std::ops::RangeInclusive;
    use std::sync::mpsc;

    use iced::widget::{column, container, text, Column};
    use iced::{Command, Element, Length, Renderer};
    use iced_aw::{Spinner, Wrap};

    use crate::core::api::series_information::{
        Genre, SeriesMainInformation, ShowNetwork, ShowWebChannel,
    };
    use crate::core::caching;
    use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};

    const SECTIONS_POSTERS_AMOUNT: usize = 20;

    const NETWORK_SECTIONS: [ShowNetwork; 7] = [
        ShowNetwork::TheCW,
        ShowNetwork::Nbc,
        ShowNetwork::Fox,
        ShowNetwork::Cbs,
        ShowNetwork::Abc,
        ShowNetwork::Hbo,
        ShowNetwork::BbcOne,
    ];

    const WEB_CHANNEL_SECTIONS: [ShowWebChannel; 1] = [ShowWebChannel::Netflix];

    const GENRE_SECTIONS: [Genre; 8] = [
        Genre::Action,
        Genre::ScienceFiction,
        Genre::Drama,
        Genre::Romance,
        Genre::Horror,
        Genre::Adventure,
        Genre::Comedy,
        Genre::Anime,
    ];

    #[derive(Debug, Clone)]
    pub enum Message {
        FullScheduleLoaded(caching::tv_schedule::full_schedule::FullSchedule),
        MonthlyNewPosters(SeriesPosterMessage),
        MonthlyReturningPosters(SeriesPosterMessage),
        PopularPosters(SeriesPosterMessage),
        NetworkPosters(SeriesPosterMessage),
        WebChannelPosters(SeriesPosterMessage),
        GenrePosters(SeriesPosterMessage),
        PosterSelected(Box<SeriesMainInformation>),
    }

    enum LoadState {
        Loading,
        Loaded,
    }

    pub struct FullSchedulePosters {
        load_state: LoadState,
        monthly_new_poster: Vec<SeriesPoster>,
        monthly_returning_posters: Vec<SeriesPoster>,
        popular_posters: Vec<SeriesPoster>,
        network_posters: Posters<ShowNetwork>,
        web_channel_posters: Posters<ShowWebChannel>,
        genre_posters: Posters<Genre>,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    }

    impl FullSchedulePosters {
        pub fn new(
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (Self, Command<Message>) {
            let command = Command::perform(
                caching::tv_schedule::full_schedule::FullSchedule::new(),
                |series| {
                    Message::FullScheduleLoaded(series.expect("failed to load series schedule"))
                },
            );
            (
                Self {
                    load_state: LoadState::Loading,
                    monthly_new_poster: vec![],
                    monthly_returning_posters: vec![],
                    popular_posters: vec![],
                    network_posters: Posters::new(),
                    web_channel_posters: Posters::new(),
                    genre_posters: Posters::new(),
                    series_page_sender,
                },
                command,
            )
        }

        pub fn update(&mut self, message: Message) -> Command<Message> {
            match message {
                Message::FullScheduleLoaded(full_schedule) => {
                    self.load_state = LoadState::Loaded;

                    let (monthly_new_posters, monthly_new_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule.get_monthly_new_series(
                                SECTIONS_POSTERS_AMOUNT,
                                get_current_month(),
                            ),
                        );

                    let (monthly_returning_posters, monthly_returning_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule.get_monthly_returning_series(
                                SECTIONS_POSTERS_AMOUNT,
                                get_current_month(),
                            ),
                        );

                    let (popular_posters, popular_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule.get_popular_series(SECTIONS_POSTERS_AMOUNT),
                        );

                    self.monthly_new_poster = monthly_new_posters;
                    self.monthly_returning_posters = monthly_returning_posters;
                    self.popular_posters = popular_posters;

                    let network_posters_commands: Vec<_> = NETWORK_SECTIONS
                        .into_iter()
                        .map(|network| {
                            let series_infos = full_schedule
                                .get_popular_series_by_network(SECTIONS_POSTERS_AMOUNT, &network);
                            self.network_posters.push_section_posters(
                                network,
                                series_infos,
                                Message::NetworkPosters,
                            )
                        })
                        .collect();

                    let genre_posters_commands: Vec<_> = GENRE_SECTIONS
                        .into_iter()
                        .map(|genre| {
                            let series_infos = full_schedule
                                .get_popular_series_by_genre(SECTIONS_POSTERS_AMOUNT, &genre);
                            self.genre_posters.push_section_posters(
                                genre,
                                series_infos,
                                Message::GenrePosters,
                            )
                        })
                        .collect();

                    let webchannel_posters_commands: Vec<_> = WEB_CHANNEL_SECTIONS
                        .into_iter()
                        .map(|webchannel| {
                            let series_infos = full_schedule.get_popular_series_by_webchannel(
                                SECTIONS_POSTERS_AMOUNT,
                                &webchannel,
                            );
                            self.web_channel_posters.push_section_posters(
                                webchannel,
                                series_infos,
                                Message::WebChannelPosters,
                            )
                        })
                        .collect();

                    Command::batch([
                        Command::batch(genre_posters_commands),
                        Command::batch(webchannel_posters_commands),
                        Command::batch(network_posters_commands),
                        Command::batch(popular_posters_commands).map(Message::PopularPosters),
                        Command::batch(monthly_returning_posters_commands)
                            .map(Message::MonthlyReturningPosters),
                        Command::batch(monthly_new_posters_commands)
                            .map(Message::MonthlyNewPosters),
                    ])
                }
                Message::MonthlyNewPosters(message) => {
                    if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                        return Command::perform(async {}, |_| {
                            Message::PosterSelected(series_information)
                        });
                    }
                    self.monthly_new_poster
                        [message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::MonthlyNewPosters)
                }
                Message::PopularPosters(message) => {
                    if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                        return Command::perform(async {}, |_| {
                            Message::PosterSelected(series_information)
                        });
                    }
                    self.popular_posters[message.get_index().expect("message should have an index")]
                        .update(message)
                        .map(Message::PopularPosters)
                }
                Message::MonthlyReturningPosters(message) => {
                    if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                        return Command::perform(async {}, |_| {
                            Message::PosterSelected(series_information)
                        });
                    }
                    self.monthly_returning_posters
                        [message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::MonthlyReturningPosters)
                }
                Message::NetworkPosters(message) => {
                    if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                        return Command::perform(async {}, |_| {
                            Message::PosterSelected(series_information)
                        });
                    }

                    self.network_posters
                        .get_series_poster_mut(
                            message.get_index().expect("message should have an index"),
                        )
                        .update(message)
                        .map(Message::NetworkPosters)
                }
                Message::WebChannelPosters(message) => {
                    if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                        return Command::perform(async {}, |_| {
                            Message::PosterSelected(series_information)
                        });
                    }
                    self.web_channel_posters
                        .get_series_poster_mut(
                            message.get_index().expect("message should have an index"),
                        )
                        .update(message)
                        .map(Message::WebChannelPosters)
                }
                Message::GenrePosters(message) => {
                    if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                        return Command::perform(async {}, |_| {
                            Message::PosterSelected(series_information)
                        });
                    }
                    self.genre_posters
                        .get_series_poster_mut(
                            message.get_index().expect("message should have and index"),
                        )
                        .update(message)
                        .map(Message::GenrePosters)
                }
                Message::PosterSelected(series_info) => {
                    self.series_page_sender
                        .send(*series_info)
                        .expect("series page receiver disconnected");
                    Command::none()
                }
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            match self.load_state {
                LoadState::Loading => container(Spinner::new())
                    .width(Length::Fill)
                    .center_x()
                    .into(),
                LoadState::Loaded => {
                    let monthly_new_posters = column![
                        text(format!(
                            "New Shows Airing in {}",
                            get_current_month().name()
                        ))
                        .size(21),
                        Wrap::with_elements(
                            self.monthly_new_poster
                                .iter()
                                .map(|poster| poster.normal_view().map(Message::MonthlyNewPosters))
                                .collect(),
                        )
                        .spacing(5.0)
                        .line_spacing(5.0)
                    ]
                    .spacing(5);

                    let monthly_returning_posters = column![
                        text(format!("Shows Returning in {}", get_current_month().name())).size(21),
                        Wrap::with_elements(
                            self.monthly_returning_posters
                                .iter()
                                .map(|poster| {
                                    poster.normal_view().map(Message::MonthlyReturningPosters)
                                })
                                .collect(),
                        )
                        .spacing(5.0)
                        .line_spacing(5.0)
                    ]
                    .spacing(5);

                    let popular_posters = column![
                        text("Popular Shows").size(21),
                        Wrap::with_elements(
                            self.popular_posters
                                .iter()
                                .map(|poster| poster.normal_view().map(Message::PopularPosters))
                                .collect()
                        )
                        .spacing(5.0)
                        .line_spacing(5.0)
                    ]
                    .spacing(5);

                    let network_sections = Column::with_children(
                        NETWORK_SECTIONS
                            .into_iter()
                            .map(|network| {
                                self.network_posters
                                    .get_section_view(&network, Message::NetworkPosters)
                            })
                            .collect(),
                    )
                    .spacing(30);

                    let genre_sections = Column::with_children(
                        GENRE_SECTIONS
                            .into_iter()
                            .map(|genre| {
                                self.genre_posters
                                    .get_section_view(&genre, Message::GenrePosters)
                            })
                            .collect(),
                    )
                    .spacing(30);

                    let webchannel_sections = Column::with_children(
                        WEB_CHANNEL_SECTIONS
                            .into_iter()
                            .map(|webchannel| {
                                self.web_channel_posters
                                    .get_section_view(&webchannel, Message::WebChannelPosters)
                            })
                            .collect(),
                    )
                    .spacing(30);

                    column![
                        popular_posters,
                        monthly_new_posters,
                        monthly_returning_posters,
                        network_sections,
                        webchannel_sections,
                        genre_sections
                    ]
                    .spacing(30)
                    .padding(10)
                    .into()
                }
            }
        }

        fn generate_posters_and_commands_from_series_infos(
            series_infos: Vec<SeriesMainInformation>,
        ) -> (Vec<SeriesPoster>, Vec<Command<SeriesPosterMessage>>) {
            let mut posters = Vec::with_capacity(series_infos.len());
            let mut posters_commands = Vec::with_capacity(series_infos.len());
            for (index, series_info) in series_infos.into_iter().enumerate() {
                let (poster, command) = SeriesPoster::new(index, series_info);
                posters.push(poster);
                posters_commands.push(command);
            }
            (posters, posters_commands)
        }
    }

    fn get_current_month() -> chrono::Month {
        use chrono::{Datelike, Local, Month};
        use num_traits::FromPrimitive;

        let current_month = Local::now().month();
        Month::from_u32(current_month).expect("current month should be valid!")
    }

    struct Posters<T> {
        index: HashMap<T, RangeInclusive<usize>>,
        posters: Vec<SeriesPoster>,
    }

    impl<T> Posters<T>
    where
        T: Eq + std::hash::Hash + std::fmt::Display,
    {
        pub fn new() -> Self {
            Self {
                index: HashMap::new(),
                posters: vec![],
            }
        }
        pub fn push_section_posters(
            &mut self,
            section_id: T,
            series_infos: Vec<SeriesMainInformation>,
            message: fn(SeriesPosterMessage) -> Message,
        ) -> Command<Message> {
            if self.posters.is_empty() {
                let range = 0..=(series_infos.len() - 1);
                let (posters, poster_commands) =
                    Self::generate_posters_and_commands_from_series_infos(&range, series_infos);
                self.index.insert(section_id, range);
                self.posters = posters;
                Command::batch(poster_commands).map(message)
            } else {
                let range = self.posters.len()..=(self.posters.len() + series_infos.len() - 1);
                let (mut posters, poster_commands) =
                    Self::generate_posters_and_commands_from_series_infos(&range, series_infos);
                self.index.insert(section_id, range);
                self.posters.append(&mut posters);
                Command::batch(poster_commands).map(message)
            }
        }

        fn generate_posters_and_commands_from_series_infos(
            range: &RangeInclusive<usize>,
            series_infos: Vec<SeriesMainInformation>,
        ) -> (Vec<SeriesPoster>, Vec<Command<SeriesPosterMessage>>) {
            assert_eq!(range.clone().count(), series_infos.len());

            let mut posters = Vec::with_capacity(series_infos.len());
            let mut posters_commands = Vec::with_capacity(series_infos.len());

            for (index, series_info) in range.clone().zip(series_infos.into_iter()) {
                let (poster, command) = SeriesPoster::new(index, series_info);
                posters.push(poster);
                posters_commands.push(command);
            }
            (posters, posters_commands)
        }

        fn get_section(&self, section_id: &T) -> &[SeriesPoster] {
            let range = self
                .index
                .get(section_id)
                .expect("section id not in the map")
                .clone();
            &self.posters[range]
        }

        pub fn get_section_view(
            &self,
            section_id: &T,
            message: fn(SeriesPosterMessage) -> Message,
        ) -> Element<'_, Message, Renderer> {
            let series_posters = self.get_section(section_id);

            let posters = Wrap::with_elements(
                series_posters
                    .iter()
                    .map(|series_poster| series_poster.normal_view().map(message))
                    .collect(),
            )
            .spacing(5.0)
            .line_spacing(5.0);

            column![text(section_id).size(21), posters]
                .spacing(5)
                .into()
        }

        pub fn get_series_poster_mut(&mut self, index: usize) -> &mut SeriesPoster {
            &mut self.posters[index]
        }
    }
}
