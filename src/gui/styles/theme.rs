use iced::theme::{Custom, Palette};
use iced::{color, Color};

#[derive(Default)]
pub enum TroxideTheme {
    #[default]
    Light,
    Dark,
}

impl TroxideTheme {
    pub fn get_theme(&self) -> Custom {
        match self {
            TroxideTheme::Light => Custom::new(Palette {
                background: color!(0xdddddd),
                text: Color::BLACK,
                primary: color!(0x8f6593),
                success: Color::from_rgb(0.0, 1.0, 0.0),
                danger: Color::from_rgb(1.0, 0.0, 0.0),
            }),
            TroxideTheme::Dark => Custom::new(Palette {
                background: color!(0x161616),
                text: color!(0xcccccc),
                primary: color!(0x8f6593),
                success: Color::from_rgb(0.0, 1.0, 0.0),
                danger: Color::from_rgb(1.0, 0.0, 0.0),
            }),
        }
    }
}
