use iced::widget::{column, container, horizontal_space, pick_list, row, text};
use iced::{Element, Renderer};

use crate::core::settings_config::{Theme, ALL_THEMES, SETTINGS};
use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
}

#[derive(Default)]
pub struct Appearance;

impl Appearance {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ThemeSelected(theme) => {
                SETTINGS.write().unwrap().change_settings().appearance.theme = theme;
            }
        }
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        let content = column![text("Appearance")
            .size(21)
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
