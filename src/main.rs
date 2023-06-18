pub mod core;
mod gui;

use iced::{Application, Settings};

fn main() -> anyhow::Result<()> {
    let config_settings = core::settings_config::load_config();
    gui::TroxideGui::run(Settings::with_flags(config_settings))?;
    Ok(())
}
