use iced::theme::Button;
use iced::Border;
use iced::{
    widget::button::{Appearance, StyleSheet},
    Background,
};

/// A custom theme that makes button transparent
pub fn transparent_button_theme() -> Button {
    Button::Custom(Box::new(TransparentButtonTheme) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

/// A custom theme that makes button transparent, with rounded border
pub fn transparent_button_with_rounded_border_theme() -> Button {
    Button::Custom(Box::new(TransparentButtonWithRoundedBorderTheme)
        as Box<dyn StyleSheet<Style = iced::Theme>>)
}

pub struct TransparentButtonTheme;

impl StyleSheet for TransparentButtonTheme {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> Appearance {
        Appearance {
            text_color: style.palette().text,
            background: Some(Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}
pub struct TransparentButtonWithRoundedBorderTheme;

impl StyleSheet for TransparentButtonWithRoundedBorderTheme {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> Appearance {
        Appearance {
            text_color: style.palette().text,
            border: Border {
                color: super::colors::accent_color(),
                width: 1.0,
                radius: 10.0.into(),
            },
            background: Some(Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}
