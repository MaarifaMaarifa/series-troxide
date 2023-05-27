use iced::{Application, Settings};

pub mod api;
mod cli;
mod database;
mod gui;

fn main() -> anyhow::Result<()> {
    // simple_logger::init()?;
    gui::Gui::run(Settings::default())?;
    Ok(())
}
