use iced::theme::Button;
use iced::{
    widget::button::{Appearance, StyleSheet},
    Background,
};

/// A custom theme that makes button transparent
pub fn transparent_button_theme() -> Button {
    Button::Custom(Box::new(TransparentButtonTheme) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

pub struct TransparentButtonTheme;

impl StyleSheet for TransparentButtonTheme {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}
