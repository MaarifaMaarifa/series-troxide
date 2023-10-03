use super::theme::TroxideTheme;
use iced::theme::Container;
use iced::widget::container::{Appearance, StyleSheet};
use iced::{color, Color};
use iced::{Background, BorderRadius};

/// A custom theme for container respecting Light and Dark TroxideTheme
pub fn first_class_container_rounded_theme() -> Container {
    Container::Custom(
        Box::new(FirstClassContainerRoundedTheme) as Box<dyn StyleSheet<Style = iced::Theme>>
    )
}

/// A custom theme for container respecting Light and Dark TroxideTheme
pub fn second_class_container_rounded_theme() -> Container {
    Container::Custom(
        Box::new(SecondClassContainerRoundedTheme) as Box<dyn StyleSheet<Style = iced::Theme>>
    )
}

/// A custom theme for container respecting Light and Dark TroxideTheme
/// designed specifically for the release time container in my_shows page
pub fn release_time_container_theme() -> Container {
    Container::Custom(
        Box::new(ContainerThemeReleaseTime) as Box<dyn StyleSheet<Style = iced::Theme>>
    )
}

/// A custom theme for container respecting Light and Dark TroxideTheme designed for the tabs
pub fn first_class_container_square_theme() -> Container {
    Container::Custom(
        Box::new(FirstClassContainerSquareTheme) as Box<dyn StyleSheet<Style = iced::Theme>>
    )
}

/// A custom theme for container respecting Light and Dark TroxideTheme designed for the tabs
pub fn second_class_container_square_theme() -> Container {
    Container::Custom(
        Box::new(SecondClassContainerSquareTheme) as Box<dyn StyleSheet<Style = iced::Theme>>
    )
}

/// A custom theme for container indicating content that represent success
pub fn success_container_theme() -> Container {
    Container::Custom(Box::new(SuccessContainerTheme) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

/// A custom theme for container indicating content that represent success
pub fn failure_container_theme() -> Container {
    Container::Custom(Box::new(FailureContainerTheme) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

/// A custom theme for container indicating content that represent loading
pub fn loading_container_theme() -> Container {
    Container::Custom(Box::new(LoadingContainerTheme) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

pub struct FirstClassContainerRoundedTheme;

impl StyleSheet for FirstClassContainerRoundedTheme {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            border_width: 1.0,
            border_radius: BorderRadius::from(10.0),
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
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

pub struct SecondClassContainerRoundedTheme;

impl StyleSheet for SecondClassContainerRoundedTheme {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            border_width: 1.0,
            border_radius: BorderRadius::from(10.0),
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
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

pub struct FirstClassContainerSquareTheme;

impl StyleSheet for FirstClassContainerSquareTheme {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            // border_width: 1.0,
            // border_radius: 10.0,
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
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

pub struct SecondClassContainerSquareTheme;

impl StyleSheet for SecondClassContainerSquareTheme {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            // border_width: 1.0,
            // border_radius: 10.0,
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
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
            border_radius: BorderRadius::from(1000.0), // Making sure it is circular
            ..Appearance::default()
        };

        match style {
            iced::Theme::Custom(custom) => {
                if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
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

pub struct SuccessContainerTheme;

impl StyleSheet for SuccessContainerTheme {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(Color {
                r: 0.0,
                g: 128_f32 / 255.0,
                b: 0.0,
                a: 0.1,
            })),
            border_color: Color {
                r: 0.0,
                g: 128_f32 / 255.0,
                b: 0.0,
                a: 1.0,
            },
            border_width: 1.0,
            border_radius: BorderRadius::from(10.0),
            ..Appearance::default()
        }
    }
}

pub struct FailureContainerTheme;

impl StyleSheet for FailureContainerTheme {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(Color {
                r: 255.0,
                g: 0.0,
                b: 0.0,
                a: 0.1,
            })),
            border_color: Color {
                r: 255.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            border_width: 1.0,
            border_radius: BorderRadius::from(10.0),
            ..Appearance::default()
        }
    }
}

pub struct LoadingContainerTheme;

impl StyleSheet for LoadingContainerTheme {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(Color {
                r: 0.0,
                g: 0.0,
                b: 0.5,
                a: 0.1,
            })),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.5,
                a: 1.0,
            },
            border_width: 1.0,
            border_radius: BorderRadius::from(10.0),
            ..Appearance::default()
        }
    }
}
