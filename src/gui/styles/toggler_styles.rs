use iced::{
    widget::toggler::{Status, Style},
    Theme,
};

use super::colors::{accent_color, gray};

pub fn always_colored_toggler_theme(_theme: &Theme, _status: Status) -> Style {
    Style {
        background: accent_color(),
        foreground: gray(),
        background_border_width: 1.0,
        background_border_color: gray(),
        foreground_border_width: 1.0,
        foreground_border_color: gray(),
    }
}
