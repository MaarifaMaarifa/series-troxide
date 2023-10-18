use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{column, scrollable};
use iced::{Alignment, Command, Element, Length, Renderer};

use crate::gui::assets::icons::GEAR_WIDE_CONNECTED;
use crate::gui::styles;
use about_widget::{About, Message as AboutMessage};
use appearance_widget::{Appearance, Message as AppearanceMessage};
use database_widget::{Database, Message as DatabaseMessage};
use discover_widget::{Discover, Message as DiscoverMessage};
use notifications_widget::{Message as NotificationsMessage, Notifications};
use settings_controls_widget::{Message as SettingsControlsMessage, SettingsControls};

use super::Tab;

mod about_widget;
mod appearance_widget;
mod database_widget;
mod discover_widget;
mod notifications_widget;
mod settings_controls_widget;

#[derive(Debug, Clone)]
pub enum Message {
    Appearance(AppearanceMessage),
    Database(DatabaseMessage),
    Notifications(NotificationsMessage),
    Discover(DiscoverMessage),
    About(AboutMessage),
    Controls(SettingsControlsMessage),
    PageScrolled(Viewport),
}

pub struct SettingsTab {
    appearance_settings: Appearance,
    database_settings: Database,
    notifications_settings: Notifications,
    discover_settings: Discover,
    about: About,
    controls_settings: SettingsControls,
    scrollable_offset: RelativeOffset,
}

impl SettingsTab {
    pub fn new() -> (Self, Command<Message>) {
        let (about_widget, about_command) = About::new();
        (
            Self {
                appearance_settings: Appearance,
                database_settings: Database::new(),
                notifications_settings: Notifications,
                discover_settings: Discover::default(),
                about: about_widget,
                scrollable_offset: RelativeOffset::START,
                controls_settings: SettingsControls,
            },
            about_command.map(Message::About),
        )
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.database_settings.subscription().map(Message::Database)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Database(message) => {
                return self
                    .database_settings
                    .update(message)
                    .map(Message::Database)
            }
            Message::Discover(message) => {
                return self
                    .discover_settings
                    .update(message)
                    .map(Message::Discover)
            }
            Message::About(message) => return self.about.update(message).map(Message::About),
            Message::Notifications(message) => self.notifications_settings.update(message),
            Message::Appearance(message) => self.appearance_settings.update(message),
            Message::Controls(message) => self.controls_settings.update(message),
            Message::PageScrolled(view_port) => {
                self.scrollable_offset = view_port.relative_offset()
            }
        }
        Command::none()
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let settings_body = scrollable(
            column![
                self.appearance_settings.view().map(Message::Appearance),
                self.database_settings.view().map(Message::Database),
                self.notifications_settings
                    .view()
                    .map(Message::Notifications),
                self.discover_settings.view().map(Message::Discover),
                self.about.view().map(Message::About),
            ]
            .spacing(10)
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .padding(5),
        )
        .id(Self::scrollable_id())
        .on_scroll(Message::PageScrolled)
        .direction(styles::scrollable_styles::vertical_direction());

        column![
            settings_body.height(Length::FillPortion(10)),
            self.controls_settings.view().map(Message::Controls)
        ]
        .align_items(Alignment::Center)
        .spacing(5)
        .into()
    }
}

impl Tab for SettingsTab {
    type Message = Message;

    fn title() -> &'static str {
        "Settings"
    }

    fn icon_bytes() -> &'static [u8] {
        GEAR_WIDE_CONNECTED
    }

    fn get_scrollable_offset(&self) -> scrollable::RelativeOffset {
        self.scrollable_offset
    }
}
