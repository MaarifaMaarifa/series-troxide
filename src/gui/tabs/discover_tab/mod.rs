use std::sync::mpsc;

use crate::core::api::series_information::{Genre, SeriesMainInformation};
use crate::core::api::updates::show_updates::*;
use crate::core::caching;
use crate::core::caching::tv_schedule::{get_series_with_country, get_series_with_date};
use crate::core::settings_config::locale_settings;
use crate::gui::assets::icons::BINOCULARS_FILL;
use crate::gui::series_page;
use crate::gui::troxide_widget;
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
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
    shows_update: LoadState,
    // series which depend on `FullSchedule`
    schedule_series: LoadState,
}

#[derive(Clone, Debug)]
pub enum Message {
    Reload,
    GlobalSeriesLoaded(Vec<SeriesMainInformation>),
    LocalSeriesLoaded(Vec<SeriesMainInformation>),
    SeriesUpdatesLoaded(Vec<SeriesMainInformation>),
    GlobalSeries(SeriesPosterMessage),
    LocalSeries(SeriesPosterMessage),
    PopularSeries(SeriesPosterMessage),
    MonthlyNewSeries(SeriesPosterMessage),
    MonthlyReturningSeries(SeriesPosterMessage),
    SeriesUpdates(SeriesPosterMessage),
    RomanceSeries(SeriesPosterMessage),
    DramaSeries(SeriesPosterMessage),
    ActionSeries(SeriesPosterMessage),
    SciFiSeries(SeriesPosterMessage),
    HorrorSeries(SeriesPosterMessage),
    AdventureSeries(SeriesPosterMessage),
    ComedySeries(SeriesPosterMessage),
    CrimeSeries(SeriesPosterMessage),
    AnimeSeries(SeriesPosterMessage),
    Search(SearchMessage),
    SeriesSelected(Box<SeriesMainInformation>),
    ShowSearchResults,
    HideSearchResults,
    EscapeKeyPressed,
    FullScheduleLoaded(caching::tv_schedule::full_schedule::FullSchedule),
}

pub struct DiscoverTab {
    load_status: LoadStatus,
    show_search_results: bool,
    search_state: searching::Search,
    series_page_sender: mpsc::Sender<(series_page::Series, Command<series_page::Message>)>,
    country_name: String,

    new_global_series: Vec<SeriesPoster>,
    new_local_series: Vec<SeriesPoster>,
    popular_series: Vec<SeriesPoster>,
    monthly_new_series: Vec<SeriesPoster>,
    monthly_returning_series: Vec<SeriesPoster>,
    romance_series: Vec<SeriesPoster>,
    action_series: Vec<SeriesPoster>,
    scifi_series: Vec<SeriesPoster>,
    drama_series: Vec<SeriesPoster>,
    horror_series: Vec<SeriesPoster>,
    comedy_series: Vec<SeriesPoster>,
    adventure_series: Vec<SeriesPoster>,
    crime_series: Vec<SeriesPoster>,
    anime_series: Vec<SeriesPoster>,
    series_updates: Vec<SeriesPoster>,
}

impl DiscoverTab {
    pub fn new(
        series_page_sender: mpsc::Sender<(series_page::Series, Command<series_page::Message>)>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                load_status: LoadStatus::default(),
                show_search_results: false,
                search_state: searching::Search::default(),
                new_global_series: vec![],
                new_local_series: vec![],
                popular_series: vec![],
                romance_series: vec![],
                scifi_series: vec![],
                drama_series: vec![],
                action_series: vec![],
                horror_series: vec![],
                adventure_series: vec![],
                comedy_series: vec![],
                crime_series: vec![],
                anime_series: vec![],
                monthly_new_series: vec![],
                monthly_returning_series: vec![],
                series_updates: vec![],
                series_page_sender,
                country_name: locale_settings::get_country_name_from_settings(),
            },
            load_discover_schedule_command(),
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
                let mut load_commands = [
                    Command::none(),
                    Command::none(),
                    Command::none(),
                    Command::none(),
                ];

                if let LoadState::Loaded = &self.load_status.local_series {
                    self.load_status.local_series = LoadState::Loading;
                    load_commands[0] = load_local_aired_series();
                }
                if let LoadState::Loaded = &self.load_status.global_series {
                    self.load_status.global_series = LoadState::Loading;
                    load_commands[1] = load_global_aired_series();
                }
                if let LoadState::Loaded = &self.load_status.shows_update {
                    self.load_status.shows_update = LoadState::Loading;
                    load_commands[2] = load_series_updates();
                }

                // `monthly new series` will represent others that obtain information
                // from `FullSchedule` since when one is loaded, all are guaranteed to be
                // loaded and vice-versa is true
                if let LoadState::Loaded = &self.load_status.schedule_series {
                    self.load_status.schedule_series = LoadState::Loading;
                    load_commands[3] = load_full_schedule();
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
            Message::SeriesUpdatesLoaded(series) => {
                self.load_status.shows_update = LoadState::Loaded;
                let mut series_posters = Vec::with_capacity(series.len());
                let mut series_poster_commands = Vec::with_capacity(series.len());
                for (index, series_info) in series.into_iter().enumerate() {
                    let (series_poster, series_poster_command) =
                        SeriesPoster::new(index, series_info);
                    series_posters.push(series_poster);
                    series_poster_commands.push(series_poster_command);
                }
                self.series_updates = series_posters;

                Command::batch(series_poster_commands).map(Message::SeriesUpdates)
            }
            Message::SeriesUpdates(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.series_updates[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::SeriesUpdates)
            }
            Message::Search(message) => {
                if let SearchMessage::SeriesResultPressed(series_info) = message {
                    self.series_page_sender
                        .send(series_page::Series::new(*series_info))
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
                    .send(series_page::Series::new(*series_info))
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
            Message::MonthlyNewSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.monthly_new_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::MonthlyNewSeries)
            }
            Message::MonthlyReturningSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.monthly_returning_series
                    [message.get_index().expect("message should have an index")]
                .update(message)
                .map(Message::MonthlyReturningSeries)
            }
            Message::PopularSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.popular_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::PopularSeries)
            }
            Message::FullScheduleLoaded(full_schedule) => {
                // Generating appropriate series posters and their commands
                let (monthly_new_posters, monthly_new_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_monthly_new_series(20, get_current_month()),
                    );

                let (monthly_returning_posters, monthly_returning_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_monthly_returning_series(20, get_current_month()),
                    );

                let (popular_posters, popular_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series(20),
                    );

                let (romance_posters, romance_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Romance),
                    );

                let (scifi_posters, scifi_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::ScienceFiction),
                    );

                let (drama_posters, drama_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Drama),
                    );

                let (horror_posters, horror_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Horror),
                    );

                let (adventure_posters, adventure_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Adventure),
                    );

                let (comedy_posters, comedy_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Comedy),
                    );

                let (crime_posters, crime_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Crime),
                    );

                let (anime_posters, anime_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Anime),
                    );

                let (action_posters, action_posters_commands) =
                    Self::generate_posters_and_commands_from_series_infos(
                        full_schedule.get_popular_series_by_genre(20, Genre::Action),
                    );

                // Finishing setting up
                self.monthly_new_series = monthly_new_posters;
                self.monthly_returning_series = monthly_returning_posters;
                self.popular_series = popular_posters;
                self.romance_series = romance_posters;
                self.action_series = action_posters;
                self.scifi_series = scifi_posters;
                self.drama_series = drama_posters;
                self.horror_series = horror_posters;
                self.adventure_series = adventure_posters;
                self.comedy_series = comedy_posters;
                self.crime_series = crime_posters;
                self.anime_series = anime_posters;

                self.load_status.schedule_series = LoadState::Loaded;

                Command::batch([
                    Command::batch(monthly_new_posters_commands).map(Message::MonthlyNewSeries),
                    Command::batch(popular_posters_commands).map(Message::PopularSeries),
                    Command::batch(monthly_returning_posters_commands)
                        .map(Message::MonthlyReturningSeries),
                    Command::batch(romance_posters_commands).map(Message::RomanceSeries),
                    Command::batch(action_posters_commands).map(Message::ActionSeries),
                    Command::batch(scifi_posters_commands).map(Message::SciFiSeries),
                    Command::batch(drama_posters_commands).map(Message::DramaSeries),
                    Command::batch(horror_posters_commands).map(Message::HorrorSeries),
                    Command::batch(adventure_posters_commands).map(Message::AdventureSeries),
                    Command::batch(comedy_posters_commands).map(Message::ComedySeries),
                    Command::batch(crime_posters_commands).map(Message::CrimeSeries),
                    Command::batch(anime_posters_commands).map(Message::AnimeSeries),
                ])
            }
            Message::RomanceSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.romance_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::RomanceSeries)
            }
            Message::ActionSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.action_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::ActionSeries)
            }
            Message::SciFiSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.scifi_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::SciFiSeries)
            }
            Message::DramaSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.drama_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::DramaSeries)
            }
            Message::HorrorSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.horror_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::HorrorSeries)
            }
            Message::AdventureSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.adventure_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::AdventureSeries)
            }
            Message::ComedySeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.comedy_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::ComedySeries)
            }
            Message::CrimeSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.crime_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::CrimeSeries)
            }
            Message::AnimeSeries(message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_information) = message {
                    self.show_search_results = false;
                    return Command::perform(async {}, |_| {
                        Message::SeriesSelected(series_information)
                    });
                }
                self.anime_series[message.get_index().expect("message should have an index")]
                    .update(message)
                    .map(Message::AnimeSeries)
            }
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
                series_posters_loader(
                    "Popular Shows",
                    &self.load_status.schedule_series,
                    &self.popular_series,
                )
                .map(Message::PopularSeries),
                series_posters_loader(
                    &format!("New Shows Airing in {} ", get_current_month().name()),
                    &self.load_status.schedule_series,
                    &self.monthly_new_series
                )
                .map(Message::MonthlyNewSeries),
                series_posters_loader(
                    &format!("Shows Returning in {}", get_current_month().name()),
                    &self.load_status.schedule_series,
                    &self.monthly_returning_series
                )
                .map(Message::MonthlyReturningSeries),
                series_posters_loader(
                    "Action",
                    &self.load_status.schedule_series,
                    &self.action_series,
                )
                .map(Message::ActionSeries),
                series_posters_loader(
                    "Science Fiction",
                    &self.load_status.schedule_series,
                    &self.scifi_series,
                )
                .map(Message::SciFiSeries),
                series_posters_loader(
                    "Drama",
                    &self.load_status.schedule_series,
                    &self.drama_series,
                )
                .map(Message::DramaSeries),
                series_posters_loader(
                    "Romance",
                    &self.load_status.schedule_series,
                    &self.romance_series,
                )
                .map(Message::RomanceSeries),
                series_posters_loader(
                    "Horror",
                    &self.load_status.schedule_series,
                    &self.horror_series,
                )
                .map(Message::HorrorSeries),
                series_posters_loader(
                    "Adventure",
                    &self.load_status.schedule_series,
                    &self.adventure_series,
                )
                .map(Message::AdventureSeries),
                series_posters_loader(
                    "Comedy",
                    &self.load_status.schedule_series,
                    &self.comedy_series,
                )
                .map(Message::ComedySeries),
                series_posters_loader(
                    "Crime",
                    &self.load_status.schedule_series,
                    &self.crime_series,
                )
                .map(Message::CrimeSeries),
                series_posters_loader(
                    "Anime",
                    &self.load_status.schedule_series,
                    &self.anime_series,
                )
                .map(Message::AnimeSeries),
                series_posters_loader(
                    "Shows Updates",
                    &self.load_status.shows_update,
                    &self.series_updates
                )
                .map(Message::SeriesUpdates),
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

    pub fn tab_label() -> troxide_widget::tabs::TabLabel {
        troxide_widget::tabs::TabLabel::new(Self::title(), BINOCULARS_FILL)
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

/// Loads series updates
fn load_series_updates() -> Command<Message> {
    Command::perform(get_show_updates(UpdateTimestamp::Day, Some(20)), |series| {
        Message::SeriesUpdatesLoaded(series.expect("failed to load series updates"))
    })
}

/// Loads the globally aired series
fn load_global_aired_series() -> Command<Message> {
    Command::perform(get_series_with_date(None), |series| {
        Message::GlobalSeriesLoaded(series.expect("failed to load series schedule"))
    })
}

fn load_full_schedule() -> Command<Message> {
    Command::perform(
        caching::tv_schedule::full_schedule::FullSchedule::new(),
        |series| Message::FullScheduleLoaded(series.expect("failed to load series schedule")),
    )
}

/// Loads series updates, globally and locally aired series all at once
fn load_discover_schedule_command() -> Command<Message> {
    Command::batch([
        load_series_updates(),
        load_global_aired_series(),
        load_local_aired_series(),
        load_full_schedule(),
    ])
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

        column!(title, vertical_space(10), wrapped_posters)
            .width(Length::Fill)
            .padding(10)
            .into()
    }
}
