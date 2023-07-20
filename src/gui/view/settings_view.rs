use iced::widget::{button, column, container, horizontal_space, pick_list, row, scrollable, text};
use iced::{Alignment, Command, Element, Length, Renderer};

use crate::core::settings_config::{save_config, Config, Theme, ALL_THEMES};
use crate::gui::assets::icons::GEAR_WIDE_CONNECTED;
use crate::gui::{styles, troxide_widget, Message as GuiMessage, Tab};
use caching_widget::{Caching, Message as CachingMessage};
use database_widget::{Database, Message as DatabaseMessage};

mod about_widget;
mod caching_widget;
mod database_widget;

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
    SaveSettings,
    Caching(CachingMessage),
    Database(DatabaseMessage),
}

#[derive(Default)]
pub struct SettingsTab {
    settings_config: Config,
    unsaved_config: Option<Config>,
    caching_settings: Caching,
    database_settings: Database,
}

impl SettingsTab {
    pub fn new(settings_config: Config) -> Self {
        Self {
            settings_config,
            unsaved_config: None,
            caching_settings: Caching::default(),
            database_settings: Database::default(),
        }
    }

    pub fn get_config_settings(&self) -> &Config {
        if let Some(config) = &self.unsaved_config {
            config
        } else {
            &self.settings_config
        }
    }
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                if let Some(config) = &mut self.unsaved_config {
                    config.appearance.theme = theme
                } else {
                    let mut unsaved_config = self.settings_config.clone();
                    unsaved_config.appearance.theme = theme;
                    self.unsaved_config = Some(unsaved_config);
                }
            }
            Message::SaveSettings => {
                if let Some(config) = self.unsaved_config.take() {
                    self.settings_config = config;
                    save_config(&self.settings_config);
                }
            }
            Message::Caching(message) => {
                return self.caching_settings.update(message).map(Message::Caching)
            }
            Message::Database(message) => {
                return self
                    .database_settings
                    .update(message)
                    .map(Message::Database)
            }
        }
        Command::none()
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let settings_body = scrollable(
            column![
                self.appearance_settings_view(),
                self.database_settings.view().map(Message::Database),
                self.caching_settings.view().map(Message::Caching),
                about_widget::about_widget(),
            ]
            .spacing(5)
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .padding(5),
        );

        let mut save_settings_button = button("Save Settings");

        if let Some(unsaved_settings) = &self.unsaved_config {
            if *unsaved_settings != self.settings_config {
                save_settings_button = save_settings_button.on_press(Message::SaveSettings);
            }
        };

        let save_button_bar = row!(horizontal_space(Length::Fill), save_settings_button).padding(5);

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
            Some(if let Some(config) = &self.unsaved_config {
                config.appearance.theme.clone()
            } else {
                self.settings_config.appearance.theme.clone()
            }),
            Message::ThemeSelected,
        );

        let content = content.push(
            row!(theme_text, horizontal_space(20), theme_picklist)
                .padding(5)
                .spacing(5),
        );

        container(content)
            .style(styles::container_styles::first_class_container_theme())
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
