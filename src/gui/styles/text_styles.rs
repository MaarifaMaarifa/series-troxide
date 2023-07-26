use iced::theme::Text;
use iced::{color, Color};

/// A custom theme that makes text purple
pub fn purple_text_theme() -> Text {
    Text::Color(color!(0x8f6593))
}

/// A custom theme that makes text red
pub fn red_text_theme() -> Text {
    Text::Color(Color::from_rgb(2.55, 0.0, 0.0))
}

/// A custom theme that makes text green
pub fn green_text_theme() -> Text {
    Text::Color(color!(0x008000))
}
