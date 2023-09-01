use iced::widget::{column, container, horizontal_space, radio, text, Column};
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

        let theme_text = text("Theme").size(18);

        let current_theme = Some(
            SETTINGS
                .read()
                .unwrap()
                .get_current_settings()
                .appearance
                .theme
                .clone(),
        );

        let theme_list = Column::with_children(
            ALL_THEMES
                .iter()
                .map(|theme| {
                    let elem: Element<'_, Message, Renderer> =
                        radio(theme.to_string(), theme, current_theme.as_ref(), |theme| {
                            Message::ThemeSelected(theme.clone())
                        })
                        .into();
                    elem
                })
                .collect(),
        )
        .spacing(5);

        let content = content.push(
            column!(theme_text, horizontal_space(20), theme_list)
                .padding(5)
                .spacing(5),
        );

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .into()
    }
}
