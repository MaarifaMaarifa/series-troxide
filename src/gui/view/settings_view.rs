use iced::widget::{button, column, container, horizontal_space, pick_list, row, scrollable, text};
use iced::{Alignment, Command, Element, Length, Renderer};

use crate::core::settings_config::{Theme, ALL_THEMES, SETTINGS};
use crate::gui::assets::icons::GEAR_WIDE_CONNECTED;
use crate::gui::{styles, troxide_widget, Message as GuiMessage, Tab};
use about_widget::{About, Message as AboutMessage};
use caching_widget::{Caching, Message as CachingMessage};
use database_widget::{Database, Message as DatabaseMessage};
use locale_widget::{Locale, Message as LocaleMessage};
use notifications_widget::{Message as NotificationsMessage, Notifications};

mod about_widget;
mod caching_widget;
mod database_widget;
mod locale_widget;
mod notifications_widget;

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
    SaveSettings,
    RestoreDefaultSettings,
    ResetSettings,
    Caching(CachingMessage),
    Database(DatabaseMessage),
    Notifications(NotificationsMessage),
    Locale(LocaleMessage),
    About(AboutMessage),
}

#[derive(Default)]
pub struct SettingsTab {
    caching_settings: Caching,
    database_settings: Database,
    notifications_settings: Notifications,
    locale_settings: Locale,
    about: About,
}

impl SettingsTab {
    pub fn new() -> Self {
        Self {
            caching_settings: Caching::default(),
            database_settings: Database::default(),
            notifications_settings: Notifications,
            locale_settings: Locale::default(),
            about: About,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                SETTINGS.write().unwrap().change_settings().appearance.theme = theme;
            }
            Message::SaveSettings => SETTINGS.write().unwrap().save_settings(),
            Message::Caching(message) => {
                return self.caching_settings.update(message).map(Message::Caching)
            }
            Message::Database(message) => {
                return self
                    .database_settings
                    .update(message)
                    .map(Message::Database)
            }
            Message::RestoreDefaultSettings => SETTINGS.write().unwrap().set_default_settings(),
            Message::ResetSettings => SETTINGS.write().unwrap().reset_settings(),
            Message::Locale(message) => self.locale_settings.update(message),
            Message::About(message) => self.about.update(message),
            Message::Notifications(message) => self.notifications_settings.update(message),
        }
        Command::none()
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let settings_body = scrollable(
            column![
                self.appearance_settings_view(),
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

        let mut save_settings_button = button("Save Settings");
        let mut reset_settings_button = button("Reset Settings");
        let mut restore_default_settings_button = button("Restore Default Settings");

        if SETTINGS.read().unwrap().has_pending_save() {
            save_settings_button = save_settings_button.on_press(Message::SaveSettings);
            reset_settings_button = reset_settings_button.on_press(Message::ResetSettings);
        }

        if !SETTINGS.read().unwrap().has_default_settings() {
            restore_default_settings_button =
                restore_default_settings_button.on_press(Message::RestoreDefaultSettings);
        }

        let save_button_bar = row![
            horizontal_space(Length::Fill),
            restore_default_settings_button,
            reset_settings_button,
            save_settings_button
        ]
        .spacing(10)
        .padding(5);

        column![
            settings_body.height(Length::FillPortion(10)),
            save_button_bar.height(Length::FillPortion(1))
        ]
        .align_items(Alignment::Center)
        .spacing(5)
        .padding(10)
        .into()
    }

    fn appearance_settings_view(&self) -> Element<Message, Renderer> {
        let content = column![text("Appearance")
            .size(25)
            .style(styles::text_styles::purple_text_theme())]
        .padding(5)
        .spacing(5);

        let theme_text = text("Theme");
        let theme_picklist = pick_list(
            &ALL_THEMES[..],
            Some(
                SETTINGS
                    .read()
                    .unwrap()
                    .get_current_settings()
                    .appearance
                    .theme
                    .clone(),
            ),
            Message::ThemeSelected,
        );

        let content = content.push(
            row!(theme_text, horizontal_space(20), theme_picklist)
                .padding(5)
                .spacing(5),
        );

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .into()
    }
}

impl Tab for SettingsTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "Settings".to_owned()
    }

    fn tab_label(&self) -> troxide_widget::tabs::TabLabel {
        troxide_widget::tabs::TabLabel::new(self.title(), GEAR_WIDE_CONNECTED)
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::Settings)
    }
}

/// A function that sleeps for 3 seconds designed to provide timeout
/// for status texts in widgets like the database and caching widget.
async fn status_timeout() {
    tokio::time::sleep(std::time::Duration::from_secs(3)).await
}
