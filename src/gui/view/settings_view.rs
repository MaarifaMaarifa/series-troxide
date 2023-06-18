use iced::widget::{pick_list, row, text};
use iced::{Element, Renderer};

use crate::core::settings_config::{Config, Theme, ALL_THEMES};

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
}

#[derive(Default)]
pub struct Settings {
    settings_config: Config,
}

impl Settings {
    pub fn new(settings_config: Config) -> Self {
        Self { settings_config }
    }

    pub fn get_config_settings(&self) -> &Config {
        &self.settings_config
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ThemeSelected(theme) => self.settings_config.theme = theme,
        }
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let theme_text = text("App Theme");
        let theme_picklist = pick_list(
            &ALL_THEMES[..],
            Some(self.settings_config.theme.clone()),
            Message::ThemeSelected,
        );

        row!(theme_text, theme_picklist)
            .padding(5)
            .spacing(5)
            .into()
    }
}
