use super::colors::accent_color;
use iced::theme::{Custom, Palette};
use iced::{color, Color};

#[derive(Default)]
pub enum TroxideTheme {
    #[default]
    Light,
    Dark,
}

impl TroxideTheme {
    pub fn get_custom_theme(&self) -> Custom {
        match self {
            TroxideTheme::Light => Custom::new(Palette {
                background: color!(0xdddddd),
                text: Color::BLACK,
                primary: accent_color(),
                success: Color::from_rgb(0.0, 1.0, 0.0),
                danger: Color::from_rgb(1.0, 0.0, 0.0),
            }),
            TroxideTheme::Dark => Custom::new(Palette {
                background: color!(0x161616),
                text: color!(0xcccccc),
                primary: accent_color(),
                success: Color::from_rgb(0.0, 1.0, 0.0),
                danger: Color::from_rgb(1.0, 0.0, 0.0),
            }),
        }
    }
}
