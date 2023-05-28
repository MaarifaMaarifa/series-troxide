pub mod core;
mod gui;

use iced::{Application, Settings};

fn main() -> anyhow::Result<()> {
    // simple_logger::init()?;
    gui::TroxideGui::run(Settings::default())?;
    Ok(())
}
