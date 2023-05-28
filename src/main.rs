use iced::{Application, Settings};

pub mod core;
mod gui;

fn main() -> anyhow::Result<()> {
    // simple_logger::init()?;
    gui::Gui::run(Settings::default())?;
    Ok(())
}
