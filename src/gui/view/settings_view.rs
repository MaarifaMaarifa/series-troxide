use iced::widget::{button, column, horizontal_space, pick_list, row, text, vertical_space};
use iced::{Element, Length, Renderer};

use crate::core::settings_config::{save_config, Config, Theme, ALL_THEMES};
use crate::gui::assets::icons::GEAR_WIDE_CONNECTED;
use crate::gui::{troxide_widget, Message as GuiMessage, Tab};

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
    SaveSettings,
}

#[derive(Default)]
pub struct SettingsTab {
    settings_config: Config,
    unsaved_config: Option<Config>,
}

impl SettingsTab {
    pub fn new(settings_config: Config) -> Self {
        Self {
            settings_config,
            unsaved_config: None,
        }
    }

    pub fn get_config_settings(&self) -> &Config {
        if let Some(config) = &self.unsaved_config {
            config
        } else {
            &self.settings_config
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ThemeSelected(theme) => {
                if let Some(config) = &mut self.unsaved_config {
                    config.theme = theme
                } else {
                    let mut unsaved_config = self.settings_config.clone();
                    unsaved_config.theme = theme;
                    self.unsaved_config = Some(unsaved_config);
                }
            }
            Message::SaveSettings => {
                if let Some(config) = self.unsaved_config.take() {
                    self.settings_config = config;
                    save_config(&self.settings_config);
                }
            }
        }
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let theme_text = text("App Theme");
        let theme_picklist = pick_list(
            &ALL_THEMES[..],
            Some(if let Some(config) = &self.unsaved_config {
                config.theme.clone()
            } else {
                self.settings_config.theme.clone()
            }),
            Message::ThemeSelected,
        );

        let settings = row!(theme_text, theme_picklist).padding(5).spacing(5);

        let save_settings_button = if let Some(unsaved_settings) = &self.unsaved_config {
            if *unsaved_settings != self.settings_config {
                button("Save Settings").on_press(Message::SaveSettings)
            } else {
                button("Save Settings")
            }
        } else {
            button("Save Settings")
        };

        let save_button_bar = row!(horizontal_space(Length::Fill), save_settings_button).padding(5);

        column!(settings, vertical_space(Length::Fill), save_button_bar).into()
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
