use iced::{widget::text, Element, Renderer};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct Watchlist;

impl Watchlist {
    pub fn view(&self) -> Element<Message, Renderer> {
        text("Watchlist View").into()
    }
}
