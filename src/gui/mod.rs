mod troxide_widget;
mod view;

use view::discover_view::Message as DiscoverMessage;
use view::menu_view::Message as MenuMessage;
use view::my_shows_view::Message as MyShowsMessage;
use view::search_view::Message as SearchMessage;
use view::series_view::Message as SeriesMessage;
use view::statistics_view::Message as StatisticsMessage;
use view::watchlist_view::Message as WatchlistMessage;

use iced::widget::row;
use iced::{Application, Command};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Message {
    MenuAction(MenuMessage),
    SearchAction(SearchMessage),
    DiscoverAction(DiscoverMessage),
    WatchlistAction(WatchlistMessage),
    MyShowsAction(MyShowsMessage),
    StatisticsAction(StatisticsMessage),
    SeriesAction(SeriesMessage),
}

#[derive(Default)]
pub struct TroxideGui {
    view: view::View,
    menu_view: view::menu_view::Menu,
    search_view: view::search_view::Search,
    discover_view: view::discover_view::Discover,
    watchlist_view: view::watchlist_view::Watchlist,
    my_shows_view: view::my_shows_view::MyShows,
    statistic_view: view::statistics_view::Statistics,
    series_view: Option<view::series_view::Series>,
}

impl Application for TroxideGui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Series Troxide".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::MenuAction(message) => {
                self.menu_view.update(message.clone());
                match message {
                    MenuMessage::Search => self.view = view::View::Search,
                    MenuMessage::Discover => {
                        self.view = view::View::Discover;
                        return self.discover_view.update(DiscoverMessage::LoadSchedule);
                    }
                    MenuMessage::Watchlist => self.view = view::View::Watchlist,
                    MenuMessage::MyShows => self.view = view::View::MyShows,
                    MenuMessage::Statistics => self.view = view::View::Statistics,
                };
                Command::none()
            }
            Message::SearchAction(message) => {
                if let SearchMessage::SeriesResultPressed(series_id) = message {
                    let (series_view, command) = view::series_view::Series::new(series_id);
                    self.series_view = Some(series_view);
                    self.view = view::View::Series;
                    return command;
                }
                self.search_view.update(message)
            }
            Message::DiscoverAction(message) => {
                if let DiscoverMessage::SeriesSelected(series_id) = message {
                    let (series_view, command) = view::series_view::Series::new(series_id);
                    self.series_view = Some(series_view);
                    self.view = view::View::Series;
                    return command;
                }
                self.discover_view.update(message)
            }
            Message::WatchlistAction(_) => todo!(),
            Message::MyShowsAction(_) => todo!(),
            Message::StatisticsAction(_) => todo!(),
            Message::SeriesAction(message) => {
                if let SeriesMessage::GoToSearchPage = message {
                    self.view = view::View::Search;
                    return Command::none();
                };
                return self
                    .series_view
                    .as_mut()
                    .expect("Series View Should be loaded")
                    .update(message);
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let menu_view = self.menu_view.view().map(Message::MenuAction);

        let main_view = match self.view {
            view::View::Search => self.search_view.view().map(Message::SearchAction),
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
        };

        row!(menu_view, main_view).into()
    }
}
