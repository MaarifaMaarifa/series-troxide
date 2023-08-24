use clap::Parser;

pub mod core;
mod gui;

use iced::{Application, Settings};

fn main() -> anyhow::Result<()> {
    let cli_command = core::cli::cli_data::Cli::parse().command;

    if let Some(command) = cli_command {
        core::cli::handle_cli::handle_cli(command)?;
        std::process::exit(0);
    }

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("starting '{}'", env!("CARGO_PKG_NAME"));

    let cache_settings = core::settings_config::SETTINGS
        .read()
        .expect("could not read the program settings")
        .get_current_settings()
        .cache
        .clone();

    tokio::runtime::Runtime::new()?
        .block_on(core::caching::cache_cleaning::auto_clean(&cache_settings))?;

    std::thread::spawn(|| core::notifications::TroxideNotify::new()?.run());

    gui::TroxideGui::run(Settings {
        default_text_size: 14.0,
        ..Default::default()
    })?;

    Ok(())
}
