use super::theme::TroxideTheme;
use iced::widget::container::{Appearance, StyleSheet};
use iced::Background;
use iced::{color, Color};

pub struct ContainerThemeFirst;

impl StyleSheet for ContainerThemeFirst {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            border_width: 1.0,
            border_radius: 10.0,
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_theme(&TroxideTheme::Light) {
                    appearance.background = Some(Background::Color(color!(0xcccccc)));
                    appearance.border_color = color!(0xbbbbbb);
                    appearance
                } else {
                    appearance.background = Some(Background::Color(color!(0x1c1c1c)));
                    appearance.border_color = Color::BLACK;
                    appearance
                }
            }
            _ => unreachable!("built-in iced themes are not in use"),
        }
    }
}

pub struct ContainerThemeSecond;

impl StyleSheet for ContainerThemeSecond {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            border_width: 1.0,
            border_radius: 10.0,
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_theme(&TroxideTheme::Light) {
                    appearance.background = Some(Background::Color(color!(0xbbbbbb)));
                    appearance.border_color = color!(0xbbbbbb);
                    appearance
                } else {
                    appearance.background = Some(Background::Color(color!(0x282828)));
                    appearance.border_color = Color::BLACK;
                    appearance
                }
            }
            _ => unreachable!("built-in iced themes are not in use"),
        }
    }
}

pub struct ContainerThemeReleaseTime;

impl StyleSheet for ContainerThemeReleaseTime {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            background: Some(Background::Color(color!(0x8f6593))),
            border_color: color!(0xbbbbbb),
            border_width: 1.0,
            border_radius: 1000.0, // Making sure it is circular
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_theme(&TroxideTheme::Light) {
                    appearance.background = Some(Background::Color(color!(0x8f6593)));
                    appearance.border_color = color!(0xbbbbbb);
                    appearance
                } else {
                    appearance.background = Some(Background::Color(color!(0x8f6593)));
                    appearance.border_color = Color::BLACK;
                    appearance
                }
            }
            _ => unreachable!("built-in iced themes are not in use"),
        }
    }
}
