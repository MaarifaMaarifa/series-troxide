use iced::{widget::text, Element, Renderer};

#[derive(Clone, Debug)]
pub enum Message {}

#[derive(Default)]
pub struct Statistics;

impl Statistics {
    pub fn view(&self) -> Element<Message, Renderer> {
        text("Statistics View").into()
    }
}
