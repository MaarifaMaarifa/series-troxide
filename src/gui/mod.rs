mod assets;
mod troxide_widget;
mod view;

use view::discover_view::Message as DiscoverMessage;
use view::menu_view::Message as MenuMessage;
use view::my_shows_view::Message as MyShowsMessage;
use view::series_view::Message as SeriesMessage;
use view::settings_view::Message as SettingsMessage;
use view::statistics_view::Message as StatisticsMessage;
use view::watchlist_view::Message as WatchlistMessage;

use iced::widget::row;
use iced::{Application, Command};

use super::core::settings_config;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Message {
    MenuAction(MenuMessage),
    DiscoverAction(DiscoverMessage),
    WatchlistAction(WatchlistMessage),
    MyShowsAction(MyShowsMessage),
    StatisticsAction(StatisticsMessage),
    SeriesAction(SeriesMessage),
    SettingsAction(SettingsMessage),
}

#[derive(Default)]
pub struct TroxideGui {
    view: view::View,
    menu_view: view::menu_view::Menu,
    discover_view: view::discover_view::Discover,
    watchlist_view: view::watchlist_view::Watchlist,
    my_shows_view: view::my_shows_view::MyShows,
    statistic_view: view::statistics_view::Statistics,
    settings_view: view::settings_view::Settings,
    series_view: Option<view::series_view::Series>,
}

impl Application for TroxideGui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = settings_config::Config;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (discover_view, discover_command) = view::discover_view::Discover::new();
        (
            Self {
                settings_view: view::settings_view::Settings::new(flags),
                discover_view: discover_view,
                ..Self::default()
            },
            discover_command,
        )
    }

    fn title(&self) -> String {
        "Series Troxide".to_string()
    }

    fn theme(&self) -> iced::Theme {
        match self.settings_view.get_config_settings().theme {
            settings_config::Theme::Light => iced::Theme::Light,
            settings_config::Theme::Dark => iced::Theme::Dark,
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::MenuAction(message) => {
                self.menu_view.update(message.clone());
                match message {
                    MenuMessage::Discover => self.view = view::View::Discover,
                    MenuMessage::Watchlist => self.view = view::View::Watchlist,
                    MenuMessage::MyShows => {
                        let (view, command) = view::my_shows_view::MyShows::new();
                        self.my_shows_view = view;
                        self.view = view::View::MyShows;
                        return command.map(Message::MyShowsAction);
                    }
                    MenuMessage::Statistics => self.view = view::View::Statistics,
                    MenuMessage::Settings => self.view = view::View::Settings,
                };
                Command::none()
            }
            Message::DiscoverAction(message) => {
                if let DiscoverMessage::SeriesSelected(series_information) = message {
                    let (series_view, command) =
                        view::series_view::Series::from_series_information(*series_information);
                    self.series_view = Some(series_view);
                    self.view = view::View::Series;
                    return command;
                }
                if let DiscoverMessage::SeriesResultSelected(series_id) = message {
                    let (series_view, command) =
                        view::series_view::Series::from_series_id(series_id);
                    self.series_view = Some(series_view);
                    self.view = view::View::Series;
                    return command;
                }
                self.discover_view.update(message)
            }
            Message::WatchlistAction(_) => todo!(),
            Message::MyShowsAction(message) => self
                .my_shows_view
                .update(message)
                .map(Message::MyShowsAction),
            Message::StatisticsAction(_) => todo!(),
            Message::SeriesAction(message) => {
                if let SeriesMessage::GoToSearchPage = message {
                    self.view = view::View::Discover;
                    return Command::none();
                };
                return self
                    .series_view
                    .as_mut()
                    .expect("Series View Should be loaded")
                    .update(message);
            }
            Message::SettingsAction(message) => {
                self.settings_view.update(message);
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let menu_view = self.menu_view.view().map(Message::MenuAction);

        let main_view = match self.view {
            view::View::Discover => self.discover_view.view().map(Message::DiscoverAction),
            view::View::MyShows => self.my_shows_view.view().map(Message::MyShowsAction),
            view::View::Statistics => self.statistic_view.view().map(Message::StatisticsAction),
            view::View::Watchlist => self.watchlist_view.view().map(Message::WatchlistAction),
            view::View::Series => self
                .series_view
                .as_ref()
                .unwrap()
                .view()
                .map(Message::SeriesAction),
            view::View::Settings => self.settings_view.view().map(Message::SettingsAction),
        };

        row!(menu_view, main_view).into()
    }
}
