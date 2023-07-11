use iced::theme::Svg;
use iced::{
    color,
    widget::svg::{Appearance, StyleSheet},
};

/// A custom theme that makes svg coloured
pub fn colored_svg_theme() -> Svg {
    Svg::Custom(Box::new(ColoredSvgTheme) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

pub struct ColoredSvgTheme;

impl StyleSheet for ColoredSvgTheme {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            color: Some(color!(0x8f6593)),
        }
    }
}
