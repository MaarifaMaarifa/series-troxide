use iced::{widget::text, Element, Renderer};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct Settings;

impl Settings {
    pub fn view(&self) -> Element<Message, Renderer> {
        text("Settings view").into()
    }
}
