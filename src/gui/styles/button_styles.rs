// use iced::theme::Button;
use iced::theme::Theme;
use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color};

/// A custom theme that makes button transparent
pub fn transparent_button_theme(theme: &Theme, _status: Status) -> Style {
    Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: theme.palette().text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// A custom theme that makes button transparent, with rounded border
pub fn transparent_button_with_rounded_border_theme(theme: &Theme, _status: Status) -> Style {
    Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: theme.palette().text,
        border: Border {
            color: super::colors::accent_color(),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}
