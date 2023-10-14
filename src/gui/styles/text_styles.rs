use super::colors::*;
use iced::theme::Text;

/// A custom theme that makes text purple
pub fn accent_color_theme() -> Text {
    Text::Color(accent_color())
}

/// A custom theme that makes text red
pub fn red_text_theme() -> Text {
    Text::Color(red())
}

/// A custom theme that makes text green
pub fn green_text_theme() -> Text {
    Text::Color(green())
}
