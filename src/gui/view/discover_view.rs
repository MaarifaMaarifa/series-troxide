use iced::{widget::text, Element, Renderer};

#[derive(Clone, Debug)]
pub enum Message {}

#[derive(Default)]
pub struct Discover;

impl Discover {
    pub fn view(&self) -> Element<Message, Renderer> {
        text("Discover View").into()
    }
}
