use iced::widget::{column, scrollable};
use iced::{Alignment, Command, Element, Length, Renderer};

use crate::gui::assets::icons::GEAR_WIDE_CONNECTED;
use about_widget::{About, Message as AboutMessage};
use appearance_widget::{Appearance, Message as AppearanceMessage};
use caching_widget::{Caching, Message as CachingMessage};
use database_widget::{Database, Message as DatabaseMessage};
use locale_widget::{Locale, Message as LocaleMessage};
use notifications_widget::{Message as NotificationsMessage, Notifications};
use settings_controls_widget::{Message as SettingsControlsMessage, SettingsControls};

mod about_widget;
mod appearance_widget;
mod caching_widget;
mod database_widget;
mod locale_widget;
mod notifications_widget;
mod settings_controls_widget;

#[derive(Debug, Clone)]
pub enum Message {
    Appearance(AppearanceMessage),
    Caching(CachingMessage),
    Database(DatabaseMessage),
    Notifications(NotificationsMessage),
    Locale(LocaleMessage),
    About(AboutMessage),
    Controls(SettingsControlsMessage),
}

#[derive(Default)]
pub struct SettingsTab {
    appearance_settings: Appearance,
    caching_settings: Caching,
    database_settings: Database,
    notifications_settings: Notifications,
    locale_settings: Locale,
    about: About,
    controls_settings: SettingsControls,
}

impl SettingsTab {
    pub fn new() -> Self {
        Self {
            appearance_settings: Appearance,
            caching_settings: Caching::default(),
            database_settings: Database::default(),
            notifications_settings: Notifications,
            locale_settings: Locale::default(),
            about: About,
            controls_settings: SettingsControls,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.database_settings.subscription().map(Message::Database)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Caching(message) => {
                return self.caching_settings.update(message).map(Message::Caching)
            }
            Message::Database(message) => {
                return self
                    .database_settings
                    .update(message)
                    .map(Message::Database)
            }
            Message::Locale(message) => self.locale_settings.update(message),
            Message::About(message) => self.about.update(message),
            Message::Notifications(message) => self.notifications_settings.update(message),
            Message::Appearance(message) => self.appearance_settings.update(message),
            Message::Controls(message) => self.controls_settings.update(message),
        }
        Command::none()
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let settings_body = scrollable(
            column![
                self.appearance_settings.view().map(Message::Appearance),
                self.database_settings.view().map(Message::Database),
                self.caching_settings.view().map(Message::Caching),
                self.notifications_settings
                    .view()
                    .map(Message::Notifications),
                self.locale_settings.view().map(Message::Locale),
                self.about.view().map(Message::About),
            ]
            .spacing(5)
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .padding(5),
        );

        column![
            settings_body.height(Length::FillPortion(10)),
            self.controls_settings.view().map(Message::Controls)
        ]
        .align_items(Alignment::Center)
        .spacing(5)
        .padding(10)
        .into()
    }
}

impl SettingsTab {
    pub fn title() -> String {
        "Settings".to_owned()
    }

    pub fn tab_label() -> super::TabLabel {
        super::TabLabel::new(Self::title(), GEAR_WIDE_CONNECTED)
    }
}

/// A function that sleeps for 3 seconds designed to provide timeout
/// for status texts in widgets like the database and caching widget.
async fn status_timeout() {
    tokio::time::sleep(std::time::Duration::from_secs(3)).await
}
