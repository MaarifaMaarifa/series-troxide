use iced::widget::{column, mouse_area, text, vertical_space};
use iced::{Element, Renderer};

#[derive(Default, Debug)]
enum MenuItem {
    #[default]
    Search,
    Discover,
    Watchlist,
    MyShows,
    Statistics,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    Search,
    Discover,
    Watchlist,
    MyShows,
    Statistics,
    Settings,
}

#[derive(Default)]
pub struct Menu {
    menu_item_selected: MenuItem,
}

impl Menu {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Discover => self.menu_item_selected = MenuItem::Discover,
            Message::Watchlist => self.menu_item_selected = MenuItem::Watchlist,
            Message::MyShows => self.menu_item_selected = MenuItem::MyShows,
            Message::Statistics => self.menu_item_selected = MenuItem::Statistics,
            Message::Search => self.menu_item_selected = MenuItem::Search,
            Message::Settings => self.menu_item_selected = MenuItem::Settings,
        }
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        column!(
            text("Series Troxide").size(25),
            vertical_space(10),
            mouse_area("Search").on_press(Message::Search),
            mouse_area("Discover").on_press(Message::Discover),
            mouse_area("Watchlist").on_press(Message::Watchlist),
            mouse_area("MyShows").on_press(Message::MyShows),
            mouse_area("Statistics").on_press(Message::Statistics),
            mouse_area("Settings").on_press(Message::Settings),
        )
        .spacing(5)
        .padding(5)
        .into()
    }
}
