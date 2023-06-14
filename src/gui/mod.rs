mod troxide_widget;
mod view;

use crate::core::api::series_information;

use view::discover_view::Message as DiscoverMessage;
use view::menu_view::Message as MenuMessage;
use view::my_shows_view::Message as MyShowsMessage;
use view::search_view::Message as SearchMessage;
use view::series_view::Message as SeriesMessage;
use view::statistics_view::Message as StatisticsMessage;
use view::watchlist_view::Message as WatchlistMessage;

use iced::widget::row;
use iced::{Application, Command};

#[derive(Debug, Clone)]
pub enum Message {
    // SeriesResultPressed(u32),
    // SeriesResultObtained((series_information::SeriesMainInformation, Option<Vec<u8>>)),
    // SeriesResultFailed,
    MenuAction(MenuMessage),
    SearchAction(SearchMessage),
    DiscoverAction(DiscoverMessage),
    WatchlistAction(WatchlistMessage),
    MyShowsAction(MyShowsMessage),
    StatisticsAction(StatisticsMessage),
    SeriesAction(SeriesMessage),
}

#[derive(Default)]
enum Page {
    #[default]
    Search,
    Series,
    Season,
    Episode,
}

#[derive(Debug)]
struct SeriesPageData {
    series_information: (series_information::SeriesMainInformation, Option<Vec<u8>>),
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
    page: Page,
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
            // Message::SeriesResultPressed(series_id) => {
            //     let series_information = series_information::get_series_main_info(series_id);

            //     Command::perform(series_information, |res| match res {
            //         Ok(res) => Message::SeriesResultObtained(res),
            //         Err(err) => {
            //             println!("Error obtaining series information: {:?}", err);
            //             Message::SeriesResultFailed
            //         }
            //     })
            // }
            // Message::SeriesResultObtained(series_information) => {
            //     // self.series_page_data = Some(SeriesPageData { series_information });
            //     self.page = Page::Series;
            //     Command::none()
            // }
            Message::MenuAction(message) => {
                self.menu_view.update(message.clone());
                match message {
                    MenuMessage::SearchPressed => self.view = view::View::Search,
                    MenuMessage::DiscoverPressed => self.view = view::View::Discover,
                    MenuMessage::WatchlistPressed => self.view = view::View::Watchlist,
                    MenuMessage::MyShowsPressed => self.view = view::View::MyShows,
                    MenuMessage::StatisticsPressed => self.view = view::View::Statistics,
                };
                return Command::none();
            }
            Message::SearchAction(message) => {
                if let SearchMessage::SeriesResultPressed(series_id) = message {
                    let (series_view, command) = view::series_view::Series::new(series_id);
                    self.series_view = Some(series_view);
                    self.view = view::View::Series;
                    return command;
                }
                return self.search_view.update(message);
            }
            // Message::SeriesResultFailed => todo!(),
            Message::DiscoverAction(_) => todo!(),
            Message::WatchlistAction(_) => todo!(),
            Message::MyShowsAction(_) => todo!(),
            Message::StatisticsAction(_) => todo!(),
            Message::SeriesAction(message) => {
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
            view::View::Menu => unreachable!("menu view should have been handled separately"),
        };

        row!(menu_view, main_view).into()
    }
}
