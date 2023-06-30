use iced::{widget::container, Element, Length, Renderer};

use crate::gui::{Message as GuiMessage, Tab};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct WatchlistTab;

impl WatchlistTab {
    pub fn view(&self) -> Element<Message, Renderer> {
        container("Watchlist View")
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Tab for WatchlistTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "Watchlist".to_owned()
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text("Watchlist icon".to_owned())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::Watchlist)
    }
}
