use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use discover_tab::{DiscoverTab, Message as DiscoverMessage};
use my_shows_tab::{Message as MyShowsMessage, MyShowsTab};
use settings_tab::{Message as SettingsMessage, SettingsTab};
use statistics_tab::{Message as StatisticsMessage, StatisticsTab};
use watchlist_tab::{Message as WatchlistMessage, WatchlistTab};

use iced::{Command, Element, Renderer};
use std::sync::mpsc;

pub mod discover_tab;
pub mod my_shows_tab;
pub mod settings_tab;
pub mod statistics_tab;
pub mod watchlist_tab;

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

pub struct TabLabel {
    pub text: String,
    pub icon: &'static [u8],
}

impl TabLabel {
    pub fn new(text: String, icon: &'static [u8]) -> Self {
        Self { text, icon }
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
}

pub struct TabsController {
    current_tab: Tab,
    discover_tab: DiscoverTab,
    settings_tab: SettingsTab,
    reloadable_tab: Option<ReloadableTab>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
}

impl TabsController {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        let (discover_tab, discover_command) = DiscoverTab::new(series_page_sender.clone());

        (
            Self {
                current_tab: Tab::Discover,
                discover_tab,
                reloadable_tab: None,
                settings_tab: SettingsTab::new(),
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
                let (statistics_tab, statistics_command) =
                    StatisticsTab::new(self.series_page_sender.clone());
                self.reloadable_tab = Some(ReloadableTab::Statistics(statistics_tab));
                statistics_command.map(Message::Statistics)
            }
            Tab::Settings => Command::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let tab_subscription = match self.current_tab {
            Tab::Discover => self.discover_tab.subscription().map(Message::Discover),
            _ => {
                if let Some(reloadable_tab) = &self.reloadable_tab {
                    match reloadable_tab {
                        ReloadableTab::MyShows(my_shows) => {
                            my_shows.subscription().map(Message::MyShows)
                        }
                        _ => iced::subscription::Subscription::none(),
                    }
                } else {
                    iced::subscription::Subscription::none()
                }
            }
        };
        iced::Subscription::batch([
            tab_subscription,
            self.settings_tab.subscription().map(Message::Settings),
        ])
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
            Message::Settings(message) => self.settings_tab.update(message).map(Message::Settings),
        }
    }

    pub fn get_labels(&self) -> Vec<TabLabel> {
        vec![
            DiscoverTab::tab_label(),
            WatchlistTab::tab_label(),
            MyShowsTab::tab_label(),
            StatisticsTab::tab_label(),
            SettingsTab::tab_label(),
        ]
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        match self.current_tab {
            Tab::Discover => self.discover_tab.view().map(Message::Discover),
            Tab::Settings => self.settings_tab.view().map(Message::Settings),
            _ => {
                let reloadable_tab = self.reloadable_tab.as_ref().expect("there must be a tab");
                match reloadable_tab {
                    ReloadableTab::Watchlist(watchlist) => watchlist.view().map(Message::Watchlist),
                    ReloadableTab::MyShows(my_shows) => my_shows.view().map(Message::MyShows),
                    ReloadableTab::Statistics(statistics) => {
                        statistics.view().map(Message::Statistics)
                    }
                }
            }
        }
    }
}
