use iced::widget::{column, mouse_area, row, svg, text, vertical_space, Row};
use iced::{Element, Length, Renderer};

use crate::gui::assets::get_static_cow_from_asset;
use crate::gui::assets::icons::{
    BINOCULARS_FILL, CARD_CHECKLIST, FILM, GEAR_WIDE_CONNECTED, GRAPH_UP_ARROW,
};

#[derive(Default, Debug)]
enum MenuItem {
    #[default]
    Discover,
    Watchlist,
    MyShows,
    Statistics,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
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
            Message::Settings => self.menu_item_selected = MenuItem::Settings,
        }
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        column!(
            text("Series Troxide").size(25),
            vertical_space(10),
            mouse_area(discover_widget()).on_press(Message::Discover),
            mouse_area(watchlist_widget()).on_press(Message::Watchlist),
            mouse_area(my_shows_widget()).on_press(Message::MyShows),
            mouse_area(statistics_widget()).on_press(Message::Statistics),
            mouse_area(settings_widget()).on_press(Message::Settings),
        )
        .spacing(5)
        .padding(5)
        .into()
    }
}

fn discover_widget() -> Row<'static, Message, Renderer> {
    let svg_handle = svg::Handle::from_memory(get_static_cow_from_asset(BINOCULARS_FILL));
    let discover_icon = svg(svg_handle).width(Length::Shrink);
    row!(discover_icon, " Discover")
}

fn watchlist_widget() -> Row<'static, Message, Renderer> {
    let svg_handle = svg::Handle::from_memory(get_static_cow_from_asset(CARD_CHECKLIST));
    let watchlist_icon = svg(svg_handle).width(Length::Shrink);
    row!(watchlist_icon, " Watchlist")
}

fn my_shows_widget() -> Row<'static, Message, Renderer> {
    let svg_handle = svg::Handle::from_memory(get_static_cow_from_asset(FILM));
    let my_shows_icon = svg(svg_handle).width(Length::Shrink);
    row!(my_shows_icon, " My Shows")
}

fn statistics_widget() -> Row<'static, Message, Renderer> {
    let svg_handle = svg::Handle::from_memory(get_static_cow_from_asset(GRAPH_UP_ARROW));
    let statistics_icon = svg(svg_handle).width(Length::Shrink);
    row!(statistics_icon, " Statistics")
}

fn settings_widget() -> Row<'static, Message, Renderer> {
    let svg_handle = svg::Handle::from_memory(get_static_cow_from_asset(GEAR_WIDE_CONNECTED));
    let settings_icon = svg(svg_handle).width(Length::Shrink);
    row!(settings_icon, " Settings")
}
