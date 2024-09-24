use iced::{
    color,
    widget::svg::{Status, Style},
    Theme,
};

/// A custom theme that makes svg coloured
pub fn colored_svg_theme(_theme: &Theme, _status: Status) -> Style {
    Style {
        color: Some(color!(0x8f6593)),
    }
}
