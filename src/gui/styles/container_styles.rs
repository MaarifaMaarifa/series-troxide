use iced::widget::container::{Appearance, StyleSheet};
use iced::Background;
use iced::{color, Color};

pub struct ContainerThemeFirst;

impl StyleSheet for ContainerThemeFirst {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(color!(0x1c1c1c))),
            border_color: Color::BLACK,
            border_width: 1.0,
            border_radius: 10.0,
            ..Appearance::default()
        }
    }
}

pub struct ContainerThemeSecond;

impl StyleSheet for ContainerThemeSecond {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(color!(0x282828))),
            border_color: Color::BLACK,
            border_width: 1.0,
            border_radius: 10.0,
            ..Appearance::default()
        }
    }
}

pub struct ContainerThemeReleaseTime;

impl StyleSheet for ContainerThemeReleaseTime {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(Color::from_rgb(0.5, 0.5, 0.0))),
            border_color: Color::BLACK,
            border_width: 1.0,
            border_radius: 1000.0, // Making sure it is circular
            ..Appearance::default()
        }
    }
}
