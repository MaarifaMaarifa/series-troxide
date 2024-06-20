use iced::{window, Application, Settings};

pub mod core;
mod gui;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    core::cli::cli_handler::handle_cli()?;

    tracing::info!("starting '{}'", env!("CARGO_PKG_NAME"));

    std::thread::spawn(|| {
        if let Err(err) = tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime")
            .block_on(core::caching::cache_updating::update_cache())
        {
            tracing::error!("failed to update cache: {}", err)
        };
    });

    std::thread::spawn(|| core::notifications::TroxideNotify::new()?.run());

    let icon = window::icon::from_file_data(gui::assets::logos::IMG_LOGO, None).ok();

    gui::TroxideGui::run(Settings {
        window: iced::window::Settings {
            icon,
            ..Default::default()
        },
        default_text_size: 14.0,
        ..Default::default()
    })?;

    Ok(())
}
