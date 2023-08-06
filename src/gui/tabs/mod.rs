use discover_view::{DiscoverTab, Message as DiscoverMessage};
use my_shows_view::{Message as MyShowsMessage, MyShowsTab};
use series_view::{Message as SeriesMessage, Series};
use settings_view::{Message as SettingsMessage, SettingsTab};
use statistics_view::{Message as StatisticsMessage, StatisticsTab};
use watchlist_view::{Message as WatchlistMessage, WatchlistTab};

use iced::{Command, Element, Renderer};
use std::sync::mpsc;

use super::troxide_widget;

pub mod discover_view;
pub mod my_shows_view;
pub mod series_view;
pub mod settings_view;
pub mod statistics_view;
pub mod watchlist_view;

#[derive(Clone)]
pub enum Tab {
    Discover,
    Watchlist,
    MyShows,
    Statistics,
    Settings,
}

impl From<usize> for Tab {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Discover,
            1 => Self::Watchlist,
            2 => Self::MyShows,
            3 => Self::Statistics,
            4 => Self::Settings,
            _ => unreachable!("no more tabs"),
        }
    }
}

impl From<Tab> for usize {
    fn from(val: Tab) -> Self {
        match val {
            Tab::Discover => 0,
            Tab::Watchlist => 1,
            Tab::MyShows => 2,
            Tab::Statistics => 3,
            Tab::Settings => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Discover(DiscoverMessage),
    Watchlist(WatchlistMessage),
    MyShows(MyShowsMessage),
    Statistics(StatisticsMessage),
    Settings(SettingsMessage),
}

enum ReloadableTab {
    Watchlist(WatchlistTab),
    MyShows(MyShowsTab),
    Statistics(StatisticsTab),
    Settings(SettingsTab),
}

pub struct TabsController {
    current_tab: Tab,
    discover_tab: DiscoverTab,
    reloadable_tab: Option<ReloadableTab>,
    series_page_sender: mpsc::Sender<(Series, Command<SeriesMessage>)>,
}

impl TabsController {
    pub fn new(
        series_page_sender: mpsc::Sender<(Series, Command<SeriesMessage>)>,
    ) -> (Self, Command<Message>) {
        let (discover_tab, discover_command) = DiscoverTab::new(series_page_sender.clone());

        (
            Self {
                current_tab: Tab::Discover,
                discover_tab,
                reloadable_tab: None,
                series_page_sender,
            },
            discover_command.map(Message::Discover),
        )
    }
    pub fn switch_to_tab(&mut self, tab: Tab) -> Command<Message> {
        self.current_tab = tab.clone();

        match tab {
            Tab::Discover => self.discover_tab.refresh().map(Message::Discover),
            Tab::Watchlist => {
                let (watchlist_tab, watchlist_command) =
                    WatchlistTab::new(self.series_page_sender.clone());
                self.reloadable_tab = Some(ReloadableTab::Watchlist(watchlist_tab));
                watchlist_command.map(Message::Watchlist)
            }
            Tab::MyShows => {
                let (my_shows_tab, my_shows_command) =
                    MyShowsTab::new(self.series_page_sender.clone());
                self.reloadable_tab = Some(ReloadableTab::MyShows(my_shows_tab));
                my_shows_command.map(Message::MyShows)
            }
            Tab::Statistics => {
                let (statistics_tab, statistics_command) = StatisticsTab::new();
                self.reloadable_tab = Some(ReloadableTab::Statistics(statistics_tab));
                statistics_command.map(Message::Statistics)
            }
            Tab::Settings => {
                self.reloadable_tab = Some(ReloadableTab::Settings(SettingsTab::new()));
                Command::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.discover_tab.subscription().map(Message::Discover)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Discover(message) => self.discover_tab.update(message).map(Message::Discover),
            Message::Watchlist(message) => {
                if let Some(ReloadableTab::Watchlist(ref mut watchlist)) = self.reloadable_tab {
                    watchlist.update(message).map(Message::Watchlist)
                } else {
                    Command::none()
                }
            }
            Message::MyShows(message) => {
                if let Some(ReloadableTab::MyShows(ref mut my_shows)) = self.reloadable_tab {
                    my_shows.update(message).map(Message::MyShows)
                } else {
                    Command::none()
                }
            }
            Message::Statistics(message) => {
                if let Some(ReloadableTab::Statistics(ref mut statistics)) = self.reloadable_tab {
                    statistics.update(message).map(Message::Statistics)
                } else {
                    Command::none()
                }
            }
            Message::Settings(message) => {
                if let Some(ReloadableTab::Settings(ref mut settings)) = self.reloadable_tab {
                    settings.update(message).map(Message::Settings)
                } else {
                    Command::none()
                }
            }
        }
    }

    pub fn get_labels(&self) -> Vec<troxide_widget::tabs::TabLabel> {
        vec![
            DiscoverTab::tab_label(),
            WatchlistTab::tab_label(),
            MyShowsTab::tab_label(),
            StatisticsTab::tab_label(),
            SettingsTab::tab_label(),
        ]
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        if let Tab::Discover = self.current_tab {
            self.discover_tab.view().map(Message::Discover)
        } else {
            let reloadable_tab = self.reloadable_tab.as_ref().expect("there must be a tab");
            match reloadable_tab {
                ReloadableTab::Watchlist(watchlist) => watchlist.view().map(Message::Watchlist),
                ReloadableTab::MyShows(my_shows) => my_shows.view().map(Message::MyShows),
                ReloadableTab::Statistics(statistics) => statistics.view().map(Message::Statistics),
                ReloadableTab::Settings(settings) => settings.view().map(Message::Settings),
            }
        }
    }
}
