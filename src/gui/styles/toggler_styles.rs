use super::colors::{accent_color, gray};
use iced::{
    theme::Toggler,
    widget::toggler::{Appearance, StyleSheet},
};

pub fn always_colored_toggler_theme() -> Toggler {
    Toggler::Custom(Box::new(AlwaysColoredStyle) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

struct AlwaysColoredStyle;

impl StyleSheet for AlwaysColoredStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, _is_active: bool) -> Appearance {
        appearance()
    }

    fn hovered(&self, _style: &Self::Style, _is_active: bool) -> Appearance {
        appearance()
    }
}
fn appearance() -> Appearance {
    Appearance {
        background: accent_color(),
        foreground: gray(),
        background_border_width: 1.0,
        background_border_color: gray(),
        foreground_border_width: 1.0,
        foreground_border_color: gray(),
    }
}
