use std::sync::mpsc;

use view::discover_view::{DiscoverTab, Message as DiscoverMessage};
use view::my_shows_view::{Message as MyShowsMessage, MyShowsTab};
use view::series_view::Message as SeriesMessage;
use view::series_view::Series;
use view::settings_view::{Message as SettingsMessage, SettingsTab};
use view::statistics_view::{Message as StatisticsMessage, StatisticsTab};
use view::watchlist_view::{Message as WatchlistMessage, WatchlistTab};

use iced::widget::{container, text, Column};
use iced::{subscription, Event};
use iced::{Application, Command, Element, Length};

use super::core::settings_config;
use crate::core::settings_config::SETTINGS;

pub mod assets;
mod helpers;
mod styles;
mod troxide_widget;
mod view;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabId {
    Discover,
    Watchlist,
    MyShows,
    Statistics,
    Settings,
}

impl From<usize> for TabId {
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

impl From<TabId> for usize {
    fn from(val: TabId) -> Self {
        match val {
            TabId::Discover => 0,
            TabId::Watchlist => 1,
            TabId::MyShows => 2,
            TabId::Statistics => 3,
            TabId::Settings => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(usize),
    Discover(DiscoverMessage),
    Watchlist(WatchlistMessage),
    MyShows(MyShowsMessage),
    Statistics(StatisticsMessage),
    Settings(SettingsMessage),
    Series(SeriesMessage),
    EventOccured(Event),
}

pub struct TroxideGui {
    active_tab: TabId,
    series_view_active: bool,
    discover_tab: DiscoverTab,
    watchlist_tab: WatchlistTab,
    my_shows_tab: MyShowsTab,
    statistics_tab: StatisticsTab,
    settings_tab: SettingsTab,
    series_view: Option<Series>,
    series_page_scroller_offset: series_page_scrolling::ScrollerOffset,
    series_page_sender: mpsc::Sender<(Series, Command<SeriesMessage>)>,
    series_page_receiver: mpsc::Receiver<(Series, Command<SeriesMessage>)>,
}

impl Application for TroxideGui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (sender, receiver) = mpsc::channel();

        let (discover_tab, discover_command) =
            view::discover_view::DiscoverTab::new(sender.clone());
        let (my_shows_tab, my_shows_command) = MyShowsTab::new(sender.clone());
        let (watchlist_tab, watchlist_command) = WatchlistTab::new(sender.clone());

        (
            Self {
                active_tab: TabId::Discover,
                series_view_active: false,
                discover_tab,
                watchlist_tab,
                statistics_tab: StatisticsTab::default(),
                my_shows_tab,
                settings_tab: view::settings_view::SettingsTab::new(),
                series_view: None,
                series_page_scroller_offset: series_page_scrolling::ScrollerOffset::default(),
                series_page_sender: sender,
                series_page_receiver: receiver,
            },
            Command::batch([
                discover_command.map(Message::Discover),
                my_shows_command.map(Message::MyShows),
                watchlist_command.map(Message::Watchlist),
            ]),
        )
    }

    fn title(&self) -> String {
        "Series Troxide".to_string()
    }

    fn theme(&self) -> iced::Theme {
        match SETTINGS
            .read()
            .unwrap()
            .get_current_settings()
            .appearance
            .theme
        {
            settings_config::Theme::Light => {
                let theme = styles::theme::TroxideTheme::Light;
                iced::Theme::Custom(Box::new(theme.get_theme()))
            }
            settings_config::Theme::Dark => {
                let theme = styles::theme::TroxideTheme::Dark;
                iced::Theme::Custom(Box::new(theme.get_theme()))
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        subscription::events().map(Message::EventOccured)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TabSelected(tab_id) => {
                self.series_view_active = false;
                let tab_id: TabId = tab_id.into();
                self.active_tab = tab_id.clone();

                if let TabId::Discover = tab_id {
                    return self.discover_tab.refresh().map(Message::Discover);
                }
                if let TabId::MyShows = tab_id {
                    let (my_shows_tab, my_shows_message) =
                        MyShowsTab::new(self.series_page_sender.clone());
                    self.my_shows_tab = my_shows_tab;
                    return my_shows_message.map(Message::MyShows);
                };
                if let TabId::Watchlist = tab_id {
                    let (watchlist_tab, watchlist_message) =
                        WatchlistTab::new(self.series_page_sender.clone());
                    self.watchlist_tab = watchlist_tab;
                    return watchlist_message.map(Message::Watchlist);
                };
                if let TabId::Statistics = tab_id {
                    return self.statistics_tab.refresh().map(Message::Statistics);
                };
                Command::none()
            }
            Message::Discover(message) => Command::batch([
                self.discover_tab.update(message).map(Message::Discover),
                self.try_series_page_switch(),
            ]),
            Message::Watchlist(message) => Command::batch([
                self.watchlist_tab.update(message).map(Message::Watchlist),
                self.try_series_page_switch(),
            ]),
            Message::MyShows(message) => Command::batch([
                self.my_shows_tab.update(message).map(Message::MyShows),
                self.try_series_page_switch(),
            ]),
            Message::Statistics(message) => {
                self.statistics_tab.update(message).map(Message::Statistics)
            }
            Message::Settings(message) => self.settings_tab.update(message).map(Message::Settings),
            Message::Series(message) => {
                if let SeriesMessage::UpdateScrollerOffset(new_value) = message {
                    self.series_page_scroller_offset.set_value(new_value);
                    return Command::none();
                }

                if let Some(command) =
                    handle_back_message_from_series(&message, &mut self.series_view_active)
                {
                    return command;
                };
                self.series_view
                    .as_mut()
                    .expect("for series view to send a message it must exist")
                    .update(message)
                    .map(Message::Series)
            }
            Message::EventOccured(event) => series_page_scrolling::handle_series_page_scrolling(
                &mut self.series_page_scroller_offset,
                event,
            ),
        }
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<Self::Theme>> {
        let mut tabs: Vec<(
            troxide_widget::tabs::TabLabel,
            Element<'_, Message, iced::Renderer>,
        )> = vec![
            (
                self.discover_tab.tab_label(),
                self.discover_tab.view().map(Message::Discover),
            ),
            (
                self.watchlist_tab.tab_label(),
                self.watchlist_tab.view().map(Message::Watchlist),
            ),
            (
                self.my_shows_tab.tab_label(),
                self.my_shows_tab.view().map(Message::MyShows),
            ),
            (
                self.statistics_tab.tab_label(),
                self.statistics_tab.view().map(Message::Statistics),
            ),
            (
                self.settings_tab.tab_label(),
                self.settings_tab.view().map(Message::Settings),
            ),
        ];

        let active_tab_index: usize = self.active_tab.to_owned().into();

        // Hijacking the current tab view when series view is active
        if self.series_view_active {
            let (_, current_view): &mut (
                troxide_widget::tabs::TabLabel,
                Element<'_, Message, iced::Renderer>,
            ) = &mut tabs[active_tab_index];
            *current_view = self
                .series_view
                .as_ref()
                .unwrap()
                .view()
                .map(Message::Series);
        }

        troxide_widget::tabs::Tabs::with_tabs(tabs, Message::TabSelected)
            .set_active_tab(active_tab_index)
            .view()
    }
}

impl TroxideGui {
    fn try_series_page_switch(&mut self) -> Command<Message> {
        match self.series_page_receiver.try_recv() {
            Ok((series_page, series_page_command)) => {
                self.series_view = Some(series_page);
                self.series_view_active = true;
                series_page_command.map(Message::Series)
            }
            Err(err) => match err {
                mpsc::TryRecvError::Empty => Command::none(),
                mpsc::TryRecvError::Disconnected => panic!("series page senders disconnected"),
            },
        }
    }
}

fn handle_back_message_from_series(
    series_message: &SeriesMessage,
    series_view_active: &mut bool,
) -> Option<Command<Message>> {
    if let SeriesMessage::GoBack = series_message {
        *series_view_active = false;
        return Some(Command::none());
    }
    None
}

trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> troxide_widget::tabs::TabLabel;

    fn view(&self) -> Element<'_, Self::Message> {
        let column = Column::new()
            .spacing(20)
            .push(text(self.title()).size(32))
            .push(self.content());

        container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn content(&self) -> Element<'_, Self::Message>;
}

mod series_page_scrolling {
    //! Deals with the scrolling in the series page
    //!
    //! Since the series page makes use of a floating element, whenever the mouse is ontop of it
    //! it captures the scrolling preventing the series page from scrolling, a behaviour that is
    //! not expected by the user when browsing the series page. This module is designed to provide
    //! a workaround by listening to mouse event and scroll the series page appropriately.

    use super::Message;
    use iced::mouse::Event as MouseEvent;
    use iced::widget::scrollable::RelativeOffset;
    use iced::widget::scrollable::{snap_to, Id};
    use iced::Command;
    use iced::Event;

    /// A custom Scroller Offset for the Scrollbar
    #[derive(Default, Debug)]
    pub struct ScrollerOffset {
        offset: f32,
    }

    impl ScrollerOffset {
        /// Puts a new value into the `ScrollerOffset` taking care the direction and numbers
        /// below 0.0 and abouve 1.0
        fn put(&mut self, value: f32) {
            let new_value = self.offset + (-1.0 * value);

            if new_value > 1.0 {
                self.offset = 1.0;
            } else if new_value < 0.0 {
                self.offset = 0.0;
            } else {
                self.offset = new_value;
            }
        }

        /// Sets the new y value in the `ScrollerOffset`, this value is expected to come from
        /// `Scroller` directly.
        pub fn set_value(&mut self, new_value: f32) {
            self.offset = new_value;
        }

        /// Get the current value of the `ScrollerOffset`
        fn get_value(&self) -> f32 {
            self.offset
        }
    }

    /// Handles the scrolling of the series page
    pub fn handle_series_page_scrolling(
        scroller_offset: &mut ScrollerOffset,
        event: Event,
    ) -> Command<Message> {
        if let Event::Mouse(MouseEvent::WheelScrolled { delta }) = event {
            match delta {
                iced::mouse::ScrollDelta::Lines { x: _, y } => {
                    scroller_offset.put(y / 10.0);
                    return snap_to(
                        Id::new("series-page-scroller"),
                        RelativeOffset {
                            x: 0.0,
                            y: scroller_offset.get_value(),
                        },
                    );
                }
                iced::mouse::ScrollDelta::Pixels { x: _, y } => {
                    scroller_offset.put(y / 10.0);
                    return snap_to(
                        Id::new("series-page-scroller"),
                        RelativeOffset {
                            x: 0.0,
                            y: scroller_offset.get_value(),
                        },
                    );
                }
            }
        }
        Command::none()
    }
}
