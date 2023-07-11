pub mod core;
mod gui;

use iced::{Application, Settings};

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("starting '{}'", env!("CARGO_PKG_NAME"));

    let config_settings = core::settings_config::load_config();
    gui::TroxideGui::run(Settings {
        flags: config_settings,
        default_font: Some(gui::assets::fonts::NOTOSANS_REGULAR_STATIC),
        ..Default::default()
    })?;
    Ok(())
}
