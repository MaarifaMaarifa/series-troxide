use iced::widget::{button, column, mouse_area, text, vertical_space};
use iced::{Element, Renderer};

#[derive(Default, Debug)]
enum MenuItem {
    #[default]
    Discover,
    Watchlist,
    MyShows,
    Statistics,
}

#[derive(Debug, Clone)]
pub enum Message {
    DiscoverPressed,
    WatchlistPressed,
    MyShowsPressed,
    StatisticsPressed,
}

#[derive(Default)]
pub struct Menu {
    menu_item_selected: MenuItem,
}

impl Menu {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::DiscoverPressed => self.menu_item_selected = MenuItem::Discover,
            Message::WatchlistPressed => self.menu_item_selected = MenuItem::Watchlist,
            Message::MyShowsPressed => self.menu_item_selected = MenuItem::MyShows,
            Message::StatisticsPressed => self.menu_item_selected = MenuItem::Statistics,
        }
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        column!(
            text("Series Troxide").size(25),
            vertical_space(10),
            mouse_area("Discover").on_press(Message::DiscoverPressed),
            mouse_area("Watchlist").on_press(Message::WatchlistPressed),
            mouse_area("MyShows").on_press(Message::MyShowsPressed),
            mouse_area("Statistics").on_press(Message::StatisticsPressed),
        )
        .spacing(5)
        .padding(5)
        .into()
    }
}
