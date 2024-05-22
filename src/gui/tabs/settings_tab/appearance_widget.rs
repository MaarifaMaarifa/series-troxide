use iced::widget::{column, container, horizontal_space, radio, slider, text, Column};
use iced::{Element, Renderer};

use crate::core::settings_config::{
    Scale, Theme, ALL_THEMES, SCALE_RANGE, SCALE_RANGE_STEP, SETTINGS,
};
use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
    ScaleSelected(Scale),
}

#[derive(Default)]
pub struct Appearance;

impl Appearance {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ThemeSelected(theme) => {
                SETTINGS.write().unwrap().change_settings().appearance.theme = theme;
            }
            Message::ScaleSelected(scale) => {
                SETTINGS.write().unwrap().change_settings().appearance.scale = scale;
            }
        }
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        let content = column![text("Appearance")
            .size(21)
            .style(styles::text_styles::accent_color_theme())]
        .padding(5)
        .spacing(5);

        let theme_text = text("Theme").size(18);

        let (current_theme, current_scale) = {
            let settings = SETTINGS
                .read()
                .unwrap()
                .get_current_settings()
                .appearance
                .to_owned();
            (Some(settings.theme), settings.scale)
        };

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

        let scale_text = text(format!("Scale {}", current_scale)).size(18);

        let scale_slider = {
            slider(SCALE_RANGE, current_scale.into(), |scale| {
                Message::ScaleSelected(scale.into())
            })
            .step(SCALE_RANGE_STEP)
        };

        let content = content
            .push(
                column!(theme_text, horizontal_space(20), theme_list)
                    .padding(5)
                    .spacing(5),
            )
            .push(
                column!(scale_text, horizontal_space(20), scale_slider)
                    .padding(5)
                    .spacing(5),
            );

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .into()
    }
}
