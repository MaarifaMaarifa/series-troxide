use iced::theme::Button;
use iced::Color;
use iced::{
    widget::button::{Appearance, StyleSheet},
    Background,
};

use super::theme::TroxideTheme;

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
        let text_color = match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
                    Color::BLACK
                } else {
                    Color::WHITE
                }
            }
            _ => unreachable!("built-in iced themes are not in use"),
        };
        Appearance {
            text_color,
            background: Some(Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}
pub struct TransparentButtonWithRoundedBorderTheme;

impl StyleSheet for TransparentButtonWithRoundedBorderTheme {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> Appearance {
        let text_color = match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
                    Color::BLACK
                } else {
                    Color::WHITE
                }
            }
            _ => unreachable!("built-in iced themes are not in use"),
        };
        Appearance {
            text_color,
            border_color: super::colors::accent_color(),
            border_radius: 10.0.into(),
            border_width: 1.0,
            background: Some(Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}
