use iced::{widget::text, Element, Renderer};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct MyShows;

impl MyShows {
    pub fn view(&self) -> Element<Message, Renderer> {
        text("MyShows View").into()
    }
}
