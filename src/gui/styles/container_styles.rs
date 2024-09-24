use super::theme::TroxideTheme;
use iced::widget::container::Style;
use iced::{color, Color, Theme};
use iced::{Background, Border};

/// A custom theme for container respecting Light and Dark TroxideTheme
pub fn first_class_container_rounded_theme(theme: &Theme) -> Style {
    let (background, border_color) = match theme {
        iced::Theme::Custom(custom) => {
            if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
                (Background::Color(color!(0xcccccc)), color!(0xbbbbbb))
            } else {
                (Background::Color(color!(0x1c1c1c)), Color::BLACK)
            }
        }
        _ => unreachable!("built-in iced themes are not in use"),
    };

    Style {
        text_color: Some(theme.palette().text),
        background: Some(background),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: border_color,
        },
        ..Default::default()
    }
}

/// A custom theme for container respecting Light and Dark TroxideTheme
pub fn second_class_container_rounded_theme(theme: &Theme) -> Style {
    let (background, border_color) = match theme {
        iced::Theme::Custom(custom) => {
            if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
                (Background::Color(color!(0xbbbbbb)), color!(0xbbbbbb))
            } else {
                (Background::Color(color!(0x282828)), Color::BLACK)
            }
        }
        _ => unreachable!("built-in iced themes are not in use"),
    };

    Style {
        text_color: Some(theme.palette().text),
        background: Some(background),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: border_color,
        },
        ..Default::default()
    }
}

/// A custom theme for container respecting Light and Dark TroxideTheme
/// designed specifically for the release time container in my_shows page
pub fn release_time_container_theme(theme: &Theme) -> Style {
    let (background, border_color) = match theme {
        iced::Theme::Custom(custom) => {
            if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
                (Background::Color(color!(0x8f6593)), color!(0xbbbbbb))
            } else {
                (Background::Color(color!(0x8f6593)), Color::BLACK)
            }
        }
        _ => unreachable!("built-in iced themes are not in use"),
    };

    Style {
        text_color: Some(theme.palette().text),
        background: Some(background),
        border: Border {
            width: 1.0,
            radius: 1000.0.into(),
            color: border_color,
        },
        ..Default::default()
    }
}

/// A custom theme for container respecting Light and Dark TroxideTheme designed for the tabs
pub fn first_class_container_square_theme(theme: &Theme) -> Style {
    let (background, border_color) = match theme {
        iced::Theme::Custom(custom) => {
            if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
                (Background::Color(color!(0xcccccc)), color!(0xbbbbbb))
            } else {
                (Background::Color(color!(0x1c1c1c)), Color::BLACK)
            }
        }
        _ => unreachable!("built-in iced themes are not in use"),
    };

    Style {
        text_color: Some(theme.palette().text),
        background: Some(background),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: border_color,
        },
        ..Default::default()
    }
}

/// A custom theme for container respecting Light and Dark TroxideTheme designed for the tabs
pub fn second_class_container_square_theme(theme: &Theme) -> Style {
    let (background, border_color) = match theme {
        iced::Theme::Custom(custom) => {
            if **custom == TroxideTheme::get_custom_theme(&TroxideTheme::Light) {
                (Background::Color(color!(0xbbbbbb)), color!(0xbbbbbb))
            } else {
                (Background::Color(color!(0x282828)), Color::BLACK)
            }
        }
        _ => unreachable!("built-in iced themes are not in use"),
    };

    Style {
        text_color: Some(theme.palette().text),
        background: Some(background),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: border_color,
        },
        ..Default::default()
    }
}

/// A custom theme for container indicating content that represent success
pub fn success_container_theme(theme: &Theme) -> Style {
    Style {
        text_color: Some(theme.palette().text),
        background: Some(Background::Color(Color {
            r: 0.0,
            g: 128_f32 / 255.0,
            b: 0.0,
            a: 0.1,
        })),
        border: Border {
            color: Color {
                r: 0.0,
                g: 128_f32 / 255.0,
                b: 0.0,
                a: 1.0,
            },
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

/// A custom theme for container indicating content that represent success
pub fn failure_container_theme(theme: &Theme) -> Style {
    Style {
        text_color: Some(theme.palette().text),
        background: Some(Background::Color(Color {
            r: 255.0,
            g: 0.0,
            b: 0.0,
            a: 0.1,
        })),
        border: Border {
            color: Color {
                r: 255.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

/// A custom theme for container indicating content that represent loading
pub fn loading_container_theme(theme: &Theme) -> Style {
    Style {
        text_color: Some(theme.palette().text),
        background: Some(Background::Color(Color {
            r: 0.0,
            g: 0.0,
            b: 0.5,
            a: 0.1,
        })),
        border: Border {
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.5,
                a: 1.0,
            },
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}
