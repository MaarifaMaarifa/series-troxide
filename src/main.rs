use iced::{window, Settings};

pub mod core;
mod gui;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("starting '{}'", env!("CARGO_PKG_NAME"));

    let db = core::database::open_database()?;

    let cache_update_db = db.clone();
    let notifications_db = db.clone();

    core::cli::cli_handler::handle_cli(db.clone())?;

    std::thread::spawn(move || {
        if let Err(err) = tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime")
            .block_on(core::caching::cache_updating::update_cache(cache_update_db))
        {
            tracing::error!("failed to update cache: {}", err)
        };
    });

    std::thread::spawn(move || core::notifications::TroxideNotify::new(notifications_db)?.run());

    // TODO: set the window icon
    let _icon = window::icon::from_file_data(gui::assets::logos::IMG_LOGO, None).ok();

    let settings = Settings {
        default_text_size: 14.0.into(),
        ..Default::default()
    };

    iced::application(
        gui::TroxideGui::title,
        gui::TroxideGui::update,
        gui::TroxideGui::view,
    )
    .subscription(gui::TroxideGui::subscription)
    .theme(gui::TroxideGui::theme)
    .scale_factor(gui::TroxideGui::scale_factor)
    .settings(settings)
    .run_with(|| gui::TroxideGui::new(db))?;

    Ok(())
}
