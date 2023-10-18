use std::sync::mpsc;

use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::gui::assets::icons::BINOCULARS_FILL;
use crate::gui::styles;
use full_schedule_posters::{FullSchedulePosters, Message as FullSchedulePostersMessage};
use searching::Message as SearchMessage;

use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{column, scrollable, Space};
use iced::{Command, Element, Length, Renderer};

use iced_aw::floating_element;

use super::Tab;

mod searching;

#[derive(Clone, Debug)]
pub enum Message {
    Reload,
    FullSchedulePosters(FullSchedulePostersMessage),
    Search(SearchMessage),
    PageScrolled(Viewport),
}

pub struct DiscoverTab {
    search: searching::Search,
    full_schedule_series: FullSchedulePosters,
    scrollable_offset: RelativeOffset,
}

impl DiscoverTab {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        let (full_schedule_series, full_schedule_command) =
            FullSchedulePosters::new(series_page_sender.clone());

        (
            Self {
                search: searching::Search::new(series_page_sender),
                full_schedule_series,
                scrollable_offset: RelativeOffset::START,
            },
            full_schedule_command.map(Message::FullSchedulePosters),
        )
    }

    pub fn refresh(&mut self) -> Command<Message> {
        self.full_schedule_series
            .refresh_daily_local_series()
            .map(Message::FullSchedulePosters)
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::subscription::events_with(|event, _| {
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                }) = event
                {
                    if key_code == iced::keyboard::KeyCode::F5 && modifiers.is_empty() {
                        return Some(Message::Reload);
                    }
                }
                None
            }),
            self.search.subscription().map(Message::Search),
        ])
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Reload => self
                .full_schedule_series
                .reload()
                .map(Message::FullSchedulePosters),
            Message::Search(message) => self.search.update(message).map(Message::Search),
            Message::FullSchedulePosters(message) => self
                .full_schedule_series
                .update(message)
                .map(Message::FullSchedulePosters),
            Message::PageScrolled(view_port) => {
                self.scrollable_offset = view_port.relative_offset();
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let underlay: Element<'_, Message, Renderer> = scrollable(
            self.full_schedule_series
                .view()
                .map(Message::FullSchedulePosters),
        )
        .direction(styles::scrollable_styles::vertical_direction())
        .id(Self::scrollable_id())
        .on_scroll(Message::PageScrolled)
        .width(Length::Fill)
        .into();

        let content = floating_element::FloatingElement::new(
            underlay,
            self.search
                .view()
                .1
                .map(|element| element.map(Message::Search))
                .unwrap_or(Space::new(0, 0).into()),
        )
        .anchor(floating_element::Anchor::North);

        column![self.search.view().0.map(Message::Search), content]
            .spacing(2)
            .into()
    }
}

impl Tab for DiscoverTab {
    type Message = Message;

    fn title() -> &'static str {
        "Discover"
    }

    fn icon_bytes() -> &'static [u8] {
        BINOCULARS_FILL
    }

    fn get_scrollable_offset(&self) -> RelativeOffset {
        self.scrollable_offset
    }
}

mod full_schedule_posters {
    use std::collections::HashMap;
    use std::ops::RangeInclusive;
    use std::sync::mpsc;

    use iced::widget::{column, container, text, vertical_space, Column};
    use iced::{Command, Element, Length, Renderer};
    use iced_aw::{Spinner, Wrap};

    use crate::core::api::tv_maze::series_information::{
        Genre, SeriesMainInformation, ShowNetwork, ShowWebChannel,
    };
    use crate::core::caching;
    use crate::core::caching::tv_schedule::full_schedule::FullSchedule;
    use crate::core::settings_config::locale_settings;
    use crate::gui::troxide_widget::series_poster::{
        IndexedMessage as SeriesPosterIndexedMessage, Message as SeriesPosterMessage, SeriesPoster,
    };

    const SECTIONS_POSTERS_AMOUNT: usize = 20;
    const DAILY_POSTERS_AMOUNT: usize = 80;

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
        FullScheduleLoaded(&'static caching::tv_schedule::full_schedule::FullSchedule),
        MonthlyNewPosters(SeriesPosterIndexedMessage<SeriesPosterMessage>),
        MonthlyReturningPosters(SeriesPosterIndexedMessage<SeriesPosterMessage>),
        GlobalSeries(SeriesPosterIndexedMessage<SeriesPosterMessage>),
        LocalSeries(SeriesPosterIndexedMessage<SeriesPosterMessage>),
        PopularPosters(SeriesPosterIndexedMessage<SeriesPosterMessage>),
        NetworkPosters(SeriesPosterIndexedMessage<SeriesPosterMessage>),
        WebChannelPosters(SeriesPosterIndexedMessage<SeriesPosterMessage>),
        GenrePosters(SeriesPosterIndexedMessage<SeriesPosterMessage>),
    }

    enum LoadState {
        Loading,
        Loaded,
    }

    pub struct FullSchedulePosters {
        load_state: LoadState,
        full_schedule: Option<&'static FullSchedule>,
        monthly_new_poster: Vec<SeriesPoster>,
        monthly_returning_posters: Vec<SeriesPoster>,
        daily_global_series: Vec<SeriesPoster>,
        daily_local_series: Vec<SeriesPoster>,
        popular_posters: Vec<SeriesPoster>,
        network_posters: Posters<ShowNetwork>,
        web_channel_posters: Posters<ShowWebChannel>,
        genre_posters: Posters<Genre>,
        country_name: String,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    }

    impl FullSchedulePosters {
        pub fn new(
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (Self, Command<Message>) {
            (
                Self {
                    load_state: LoadState::Loading,
                    full_schedule: None,
                    monthly_new_poster: vec![],
                    monthly_returning_posters: vec![],
                    daily_global_series: vec![],
                    daily_local_series: vec![],
                    popular_posters: vec![],
                    network_posters: Posters::new(series_page_sender.clone()),
                    web_channel_posters: Posters::new(series_page_sender.clone()),
                    genre_posters: Posters::new(series_page_sender.clone()),
                    country_name: locale_settings::get_country_name_from_settings(),
                    series_page_sender,
                },
                Self::load_full_schedule(),
            )
        }

        pub fn reload(&mut self) -> Command<Message> {
            if let LoadState::Loaded = self.load_state {
                self.load_state = LoadState::Loading;
                Self::load_full_schedule()
            } else {
                Command::none()
            }
        }

        pub fn refresh_daily_local_series(&mut self) -> Command<Message> {
            let current_country_name = locale_settings::get_country_name_from_settings();

            if self.country_name != current_country_name {
                if let Some(full_schedule) = self.full_schedule {
                    let country_code = locale_settings::get_country_code_from_settings();

                    let (daily_local_posters, daily_local_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule
                                .get_daily_local_series(DAILY_POSTERS_AMOUNT, &country_code),
                            self.series_page_sender.clone(),
                        );

                    self.daily_local_series = daily_local_posters;
                    self.country_name = current_country_name;

                    Command::batch(daily_local_posters_commands).map(Message::LocalSeries)
                } else {
                    Command::none()
                }
            } else {
                Command::none()
            }
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
                            self.series_page_sender.clone(),
                        );

                    let (monthly_returning_posters, monthly_returning_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule.get_monthly_returning_series(
                                SECTIONS_POSTERS_AMOUNT,
                                get_current_month(),
                            ),
                            self.series_page_sender.clone(),
                        );

                    let (popular_posters, popular_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule.get_popular_series(SECTIONS_POSTERS_AMOUNT),
                            self.series_page_sender.clone(),
                        );

                    let (daily_global_posters, daily_global_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule.get_daily_global_series(DAILY_POSTERS_AMOUNT),
                            self.series_page_sender.clone(),
                        );

                    let (daily_local_posters, daily_local_posters_commands) =
                        Self::generate_posters_and_commands_from_series_infos(
                            full_schedule.get_daily_local_series(
                                DAILY_POSTERS_AMOUNT,
                                &locale_settings::get_country_code_from_settings(),
                            ),
                            self.series_page_sender.clone(),
                        );

                    self.full_schedule = Some(full_schedule);
                    self.monthly_new_poster = monthly_new_posters;
                    self.monthly_returning_posters = monthly_returning_posters;
                    self.popular_posters = popular_posters;
                    self.daily_global_series = daily_global_posters;
                    self.daily_local_series = daily_local_posters;

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
                        Command::batch(daily_global_posters_commands).map(Message::GlobalSeries),
                        Command::batch(daily_local_posters_commands).map(Message::LocalSeries),
                    ])
                }
                Message::MonthlyNewPosters(message) => self.monthly_new_poster[message.index()]
                    .update(message)
                    .map(Message::MonthlyNewPosters),
                Message::PopularPosters(message) => self.popular_posters[message.index()]
                    .update(message)
                    .map(Message::PopularPosters),
                Message::MonthlyReturningPosters(message) => self.monthly_returning_posters
                    [message.index()]
                .update(message)
                .map(Message::MonthlyReturningPosters),
                Message::NetworkPosters(message) => self
                    .network_posters
                    .get_series_poster_mut(message.index())
                    .update(message)
                    .map(Message::NetworkPosters),
                Message::WebChannelPosters(message) => self
                    .web_channel_posters
                    .get_series_poster_mut(message.index())
                    .update(message)
                    .map(Message::WebChannelPosters),
                Message::GenrePosters(message) => self
                    .genre_posters
                    .get_series_poster_mut(message.index())
                    .update(message)
                    .map(Message::GenrePosters),
                Message::GlobalSeries(message) => self.daily_global_series[message.index()]
                    .update(message)
                    .map(Message::GlobalSeries),
                Message::LocalSeries(message) => self.daily_local_series[message.index()]
                    .update(message)
                    .map(Message::LocalSeries),
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            match self.load_state {
                LoadState::Loading => container(Spinner::new())
                    .width(Length::Fill)
                    .height(500)
                    .center_x()
                    .center_y()
                    .into(),
                LoadState::Loaded => {
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
                        series_posters_viewer(
                            "Shows Airing Today Globally",
                            &self.daily_global_series
                        )
                        .map(Message::GlobalSeries),
                        series_posters_viewer(
                            &format!("Shows Airing Today in {}", self.country_name),
                            &self.daily_local_series
                        )
                        .map(Message::LocalSeries),
                        series_posters_viewer("Popular Shows", &self.popular_posters)
                            .map(Message::PopularPosters),
                        series_posters_viewer(
                            &format!("New Shows Airing in {}", get_current_month().name()),
                            &self.monthly_new_poster,
                        )
                        .map(Message::MonthlyNewPosters),
                        series_posters_viewer(
                            &format!("Shows Returning in {}", get_current_month().name()),
                            &self.monthly_returning_posters,
                        )
                        .map(Message::MonthlyReturningPosters),
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

        fn load_full_schedule() -> Command<Message> {
            Command::perform(
                caching::tv_schedule::full_schedule::FullSchedule::new(),
                |series| {
                    Message::FullScheduleLoaded(series.expect("failed to load series schedule"))
                },
            )
        }

        fn generate_posters_and_commands_from_series_infos(
            series_infos: Vec<SeriesMainInformation>,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (
            Vec<SeriesPoster>,
            Vec<Command<SeriesPosterIndexedMessage<SeriesPosterMessage>>>,
        ) {
            let mut posters = Vec::with_capacity(series_infos.len());
            let mut posters_commands = Vec::with_capacity(series_infos.len());
            for (index, series_info) in series_infos.into_iter().enumerate() {
                let (poster, command) =
                    SeriesPoster::new(index, series_info, series_page_sender.clone());
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

    /// Show `No Series Found` information in a discover section
    fn no_series_found() -> Element<'static, Message, Renderer> {
        container(text("No Series Found"))
            .center_x()
            .center_y()
            .height(100)
            .width(Length::Fill)
            .into()
    }

    fn series_posters_viewer<'a>(
        title: &str,
        posters: &'a [SeriesPoster],
    ) -> Element<'a, SeriesPosterIndexedMessage<SeriesPosterMessage>, Renderer> {
        let title = text(title).size(21);

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
            let wrapped_posters = Wrap::with_elements(
                posters
                    .iter()
                    .filter(|poster| !poster.is_hidden())
                    .map(|poster| poster.normal_view(true))
                    .collect(),
            )
            .spacing(5.0)
            .line_spacing(5.0);

            column!(title, wrapped_posters)
                .spacing(5)
                .width(Length::Fill)
                .into()
        }
    }
    struct Posters<T> {
        index: HashMap<T, RangeInclusive<usize>>,
        posters: Vec<SeriesPoster>,

        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    }

    impl<T> Posters<T>
    where
        T: Eq + std::hash::Hash + std::fmt::Display,
    {
        pub fn new(series_page_sender: mpsc::Sender<SeriesMainInformation>) -> Self {
            Self {
                index: HashMap::new(),
                posters: vec![],
                series_page_sender,
            }
        }
        pub fn push_section_posters(
            &mut self,
            section_id: T,
            series_infos: Vec<SeriesMainInformation>,
            message: fn(SeriesPosterIndexedMessage<SeriesPosterMessage>) -> Message,
        ) -> Command<Message> {
            if self.posters.is_empty() {
                let range = 0..=(series_infos.len() - 1);
                let (posters, poster_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        &range,
                        series_infos,
                        self.series_page_sender.clone(),
                    );
                self.index.insert(section_id, range);
                self.posters = posters;
                Command::batch(poster_commands).map(message)
            } else {
                let range = self.posters.len()..=(self.posters.len() + series_infos.len() - 1);
                let (mut posters, poster_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        &range,
                        series_infos,
                        self.series_page_sender.clone(),
                    );
                self.index.insert(section_id, range);
                self.posters.append(&mut posters);
                Command::batch(poster_commands).map(message)
            }
        }

        fn generate_posters_and_commands_from_series_infos(
            range: &RangeInclusive<usize>,
            series_infos: Vec<SeriesMainInformation>,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (
            Vec<SeriesPoster>,
            Vec<Command<SeriesPosterIndexedMessage<SeriesPosterMessage>>>,
        ) {
            assert_eq!(range.clone().count(), series_infos.len());

            let mut posters = Vec::with_capacity(series_infos.len());
            let mut posters_commands = Vec::with_capacity(series_infos.len());

            for (index, series_info) in range.clone().zip(series_infos.into_iter()) {
                let (poster, command) =
                    SeriesPoster::new(index, series_info, series_page_sender.clone());
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
            message: fn(SeriesPosterIndexedMessage<SeriesPosterMessage>) -> Message,
        ) -> Element<'_, Message, Renderer> {
            let series_posters = self.get_section(section_id);

            let posters: Element<'_, Message, Renderer> = if series_posters.is_empty() {
                no_series_found()
            } else {
                Wrap::with_elements(
                    series_posters
                        .iter()
                        .filter(|poster| !poster.is_hidden())
                        .map(|series_poster| series_poster.normal_view(true).map(message))
                        .collect(),
                )
                .spacing(5.0)
                .line_spacing(5.0)
                .into()
            };

            column![text(section_id).size(21), posters]
                .spacing(5)
                .into()
        }

        pub fn get_series_poster_mut(&mut self, index: usize) -> &mut SeriesPoster {
            &mut self.posters[index]
        }
    }
}
